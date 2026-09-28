use serde::{Deserialize, Serialize};

use crate::mc::McVoice;
use crate::medley::MedleySlot;
use crate::room::RemoteConfig;
use crate::tip::TipConfig;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub code: String,
    pub title: String,
    pub artist: String,
    /// Other spellings people search by, e.g. the channel's romanised title ("Rak Mai Wai Laew Voi").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub youtube_id: String,
    #[serde(default)]
    pub guide: Option<GuideTrack>,
    pub duration_secs: u32,
    pub intro_skip_secs: u32,
    pub category: String,
    pub channel: String,
    pub is_favorite: bool,
}

impl Song {
    /// Karaoke video second playback begins at, depending on whether the intro is skipped.
    pub fn start_sec(&self, skip_intro: bool) -> u64 {
        match skip_intro {
            true => u64::from(self.intro_skip_secs),
            false => 0,
        }
    }
}

/// Original-singer audio played in sync under the muted karaoke video.
/// Mapping: MV time = offset_secs + rate * karaoke time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuideTrack {
    pub video_id: String,
    pub offset_secs: f32,
    pub rate: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueueItem {
    pub queue_id: u64,
    pub song: Song,
    pub requester: String,
    /// One part of a medley: played from its start to its end, then the next part follows (plan 004).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<MedleySlot>,
}

impl QueueItem {
    /// Karaoke second this entry plays from: a medley part's start, else the song's (with or without its intro).
    pub fn start_at(&self, skip_intro: bool) -> f64 {
        match &self.part {
            Some(slot) => slot.span.start,
            None => self.song.start_sec(skip_intro) as f64,
        }
    }

    /// A later part of a medley, which follows the one before straight on.
    pub fn is_medley_join(&self) -> bool {
        self.part.as_ref().is_some_and(MedleySlot::is_join)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KtvTab {
    Catalog,
    Queue,
    Remote,
    CustomAdd,
    Settings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_intro_skip_secs: u32,
    pub auto_skip_intro: bool,
    pub volume: u32,
    pub room_name: String,
    pub sound_fx_enabled: bool,
    /// Curator tools: line up original-vocal MVs by ear (added later, so older saved settings lack it).
    #[serde(default)]
    pub show_timing_tools: bool,
    /// The shortcut help was closed once; until then it opens on every visit.
    #[serde(default)]
    pub seen_shortcuts: bool,
    /// Booth / TV screen: larger UI, wider player, "Up next" under the video.
    #[serde(default)]
    pub tv_mode: bool,
    /// Plan 003: Solana Pay tip QR under the player (off until a wallet is set).
    #[serde(default)]
    pub tip: TipConfig,
    /// Plan 003 A3: spoken MC between songs (off until chosen).
    #[serde(default)]
    pub mc_voice: McVoice,
    /// Phone remote: guests queue songs from `/remote` (off until the host turns it on in the Keypad tab).
    #[serde(default)]
    pub remote: RemoteConfig,
    /// Duet: two mics on one stereo receiver (left = singer 1, right = singer 2), each scored on its own.
    #[serde(default)]
    pub duet: bool,
}

impl AppSettings {
    /// Booth volume 0-100 (a hand-edited save above 100 counts as 100).
    pub fn volume(&self) -> u32 {
        self.volume.min(100)
    }

    /// Change the volume by `delta` points, kept within 0-100; returns the new volume.
    pub fn volume_by(&mut self, delta: i32) -> u32 {
        self.volume = self.volume().saturating_add_signed(delta).min(100);
        self.volume
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_intro_skip_secs: 18,
            auto_skip_intro: true,
            volume: 85,
            room_name: "VIP ROOM 07".to_string(),
            sound_fx_enabled: true,
            show_timing_tools: false,
            seen_shortcuts: false,
            tv_mode: false,
            tip: TipConfig::default(),
            mc_voice: McVoice::Off,
            remote: RemoteConfig::default(),
            duet: false,
        }
    }
}
