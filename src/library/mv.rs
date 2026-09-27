use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

use crate::types::GuideTrack;

/// `assets/mv_guides.json` (from `tools/match_mv.py`): the official original-vocal video for library songs, keyed by
/// the karaoke video id. Small, so compiled in; `cargo test` rejects a malformed file.
pub const MV_GUIDES_JSON: &str = include_str!("../../assets/mv_guides.json");

/// One entry: only `video_id` while it is a suggestion; `offset_secs` (and `rate`) once lined up, by a curator or,
/// with `auto`, from the official audio track's fixed offset (tools/official_audio.py).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MvGuide {
    pub video_id: String,
    pub offset_secs: Option<f32>,
    pub rate: Option<f32>,
    #[serde(default)]
    pub auto: bool,
}

pub static MV_GUIDES: LazyLock<HashMap<String, MvGuide>> =
    LazyLock::new(|| serde_json::from_str(MV_GUIDES_JSON).expect("assets/mv_guides.json must match MvGuide entries"));

/// The lined-up guide for a library song's karaoke video; `None` until a curator has timed its MV.
pub fn timed_guide(karaoke_id: &str) -> Option<GuideTrack> {
    let guide = MV_GUIDES.get(karaoke_id)?;
    Some(GuideTrack { video_id: guide.video_id.clone(), offset_secs: guide.offset_secs?, rate: guide.rate.unwrap_or(1.0) })
}

/// The song's guide was timed automatically (official audio track), not by ear.
pub fn auto_timed(karaoke_id: &str) -> bool {
    MV_GUIDES.get(karaoke_id).is_some_and(|guide| guide.auto && guide.offset_secs.is_some())
}

/// The matched official video still waiting to be timed (offered in the Guide Timing Tools).
pub fn suggested_video(karaoke_id: &str) -> Option<&'static str> {
    let guide = MV_GUIDES.get(karaoke_id)?;
    guide.offset_secs.is_none().then_some(guide.video_id.as_str())
}
