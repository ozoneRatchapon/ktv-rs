use super::types::{MicChannels, MicError};

/// Native builds (tests, desktop) have no browser mic; `start` always fails.
pub struct Mic;

impl Mic {
    pub async fn start<F, H>(_channels: MicChannels, _make_handler: F) -> Result<Self, MicError>
    where
        F: FnOnce(f32) -> H,
        H: FnMut(usize, &[f32]) + 'static,
    {
        Err(MicError::Unsupported)
    }
}
