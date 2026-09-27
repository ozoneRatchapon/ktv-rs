//! Plan 003 A3: a spoken MC from templates, voiced by the browser's own speech synthesis ($0, on-device).

mod lines;
mod types;

pub use lines::{mc_line, seed_of};
pub use types::{McEvent, McVoice};

use crate::browser;

/// Say the MC's line for `event` now, if the MC is on and the device has a local voice for its language.
pub fn announce(event: &McEvent, voice: McVoice, seed: u64) {
    if let (Some(line), Some(lang)) = (mc_line(event, voice, seed), voice.lang()) {
        if !browser::speak(&line, lang) {
            dioxus::logger::tracing::info!("MC: no on-device {lang} voice, staying silent");
        }
    }
}
