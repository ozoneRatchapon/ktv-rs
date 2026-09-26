//! In-app guide timing: line an original-vocal MV up with a karaoke video by ear, using only the
//! embedded players (no audio download). Pure math here; the UI lives in `components::guide_timing`.

mod fit;
mod types;

pub use fit::{apply_overrides, catalog_snippet, fit, mark_at, new_guide, nudge, MIN_FIT_SPAN_SECS, RATE_RANGE};
pub use types::{FitError, GuideOverrides, SyncMark};
