use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::types::GuideTrack;

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

/// `assets/mv_guides.json` (from `tools/match_mv.py`): the official original-vocal video for library songs, keyed by
/// the karaoke video id. ≈115 KB gzip (mostly incompressible video ids), so fetched with the library, not compiled in.
static MV_GUIDES: OnceLock<HashMap<String, MvGuide>> = OnceLock::new();

/// Parse and keep the guides for the rest of the page's life; a second call is ignored. Install before the library,
/// whose songs take their timed guide from here.
pub fn install_guides(json: &str) -> Result<(), serde_json::Error> {
    let guides = serde_json::from_str(json)?;
    MV_GUIDES.get_or_init(|| guides);
    Ok(())
}

/// Every guide once installed, else `None`.
pub fn guides() -> Option<&'static HashMap<String, MvGuide>> {
    MV_GUIDES.get()
}

fn guide(karaoke_id: &str) -> Option<&'static MvGuide> {
    guides()?.get(karaoke_id)
}

/// The lined-up guide for a library song's karaoke video; `None` until a curator has timed its MV.
pub fn timed_guide(karaoke_id: &str) -> Option<GuideTrack> {
    let guide = guide(karaoke_id)?;
    Some(GuideTrack { video_id: guide.video_id.clone(), offset_secs: guide.offset_secs?, rate: guide.rate.unwrap_or(1.0) })
}

/// The song's guide was timed automatically (official audio track), not by ear.
pub fn auto_timed(karaoke_id: &str) -> bool {
    guide(karaoke_id).is_some_and(|guide| guide.auto && guide.offset_secs.is_some())
}

/// The matched official video still waiting to be timed (offered in the Guide Timing Tools).
pub fn suggested_video(karaoke_id: &str) -> Option<&'static str> {
    let guide = guide(karaoke_id)?;
    guide.offset_secs.is_none().then_some(guide.video_id.as_str())
}
