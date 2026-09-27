//! Plan 003 A3: the MC's lines, from templates (no LLM, no network). A few wordings per event, picked by a
//! seed (the song), so the same song always gets the same line and a night does not sound like one loop.
//! Thai lines leave out ครับ / ค่ะ: the MC has no gender.

use super::types::{McEvent, McVoice};

const SONG_UP_TH: [&str; 3] = ["เพลงต่อไป {title} ของ {artist}", "ขอเสียงปรบมือให้เพลง {title} ของ {artist}", "ไมค์พร้อมแล้ว {title} จาก {artist}"];
const SONG_UP_EN: [&str; 3] = ["Next up: {title} by {artist}!", "Give it up for {title}, by {artist}!", "Mic's ready: {title}, from {artist}."];
const TIPPED_TH: &str = "เพลงขอพิเศษพร้อมทิป! {title} ของ {artist} มาแล้ว";
const TIPPED_EN: &str = "A tipped request! {title} by {artist}, coming right up.";

/// What the MC says for `event`, or `None` when the MC is off.
pub fn mc_line(event: &McEvent, voice: McVoice, seed: u64) -> Option<String> {
    let thai = match voice {
        McVoice::Off => return None,
        McVoice::Thai => true,
        McVoice::English => false,
    };
    let line = match event {
        McEvent::SongUp { title, artist, tipped } => {
            let template = match (tipped, thai) {
                (true, true) => TIPPED_TH,
                (true, false) => TIPPED_EN,
                (false, true) => pick(&SONG_UP_TH, seed),
                (false, false) => pick(&SONG_UP_EN, seed),
            };
            template.replace("{title}", title).replace("{artist}", artist)
        }
        McEvent::TakeEnded { score, singer } => {
            let verdict = take_verdict(*score, thai);
            match singer {
                // "Nok, spot on!": the verdict continues the sentence (Thai has no case)
                Some(name) => {
                    let mut chars = verdict.chars();
                    let first = chars.next().map(|c| c.to_lowercase().to_string()).unwrap_or_default();
                    format!("{name}, {first}{}", chars.as_str())
                }
                None => verdict,
            }
        }
    };
    Some(line)
}

/// Same bands as the score card (90 / 70 / 40).
fn take_verdict(score: u8, thai: bool) -> String {
    match (score, thai) {
        (90.., true) => format!("เป๊ะมาก! {score} คะแนน"),
        (90.., false) => format!("Spot on! {score} points!"),
        (70..=89, true) => format!("ร้องได้ดีเลย {score} คะแนน"),
        (70..=89, false) => format!("Nicely in tune: {score} points."),
        (40..=69, true) => format!("ใกล้แล้ว {score} คะแนน เอาอีกเพลงไหม"),
        (40..=69, false) => format!("Getting there: {score} points. One more?"),
        (_, true) => "ฝึกอีกนิด สู้ๆ".to_string(),
        (_, false) => "Keep practising, you've got this!".to_string(),
    }
}

fn pick<'a>(options: &[&'a str], seed: u64) -> &'a str {
    options[(seed % options.len() as u64) as usize]
}

/// A stable seed from text (FNV-1a), so a song keeps its line across reloads and builds.
pub fn seed_of(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3))
}
