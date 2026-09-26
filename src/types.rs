use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Song {
    pub id: String,
    pub code: String,
    pub title: String,
    pub artist: String,
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
    pub key_shift: i32,
    pub requester: String,
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
        }
    }
}
