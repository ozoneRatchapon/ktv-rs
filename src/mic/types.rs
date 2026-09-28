/// Analysis frame length in samples (~43 ms at 48 kHz: two periods of the 80 Hz floor).
pub const FRAME_SIZE: u32 = 2048;
/// New frame every this many samples (50% overlap, ~47 frames/s at 48 kHz).
pub const HOP_SIZE: u32 = 1024;

/// How many voices one input device carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MicChannels {
    /// One singer. Echo cancellation on: it removes this tab's backing track from the mic.
    #[default]
    Mono,
    /// Duet on a stereo receiver (two wireless mics: left = singer 1, right = singer 2). Echo cancellation off,
    /// because browsers mix it down to mono; speakers leaking into the mics can move the scores.
    Stereo,
}

impl MicChannels {
    pub fn count(self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }
}

/// Why the microphone could not be opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MicError {
    /// Not a browser, or no `getUserMedia` / AudioWorklet (e.g. an insecure http origin).
    Unsupported,
    /// The user or browser policy refused mic access.
    PermissionDenied,
    /// No input device present.
    NoDevice,
    /// The device exists but could not be started (in use elsewhere, hardware error).
    DeviceBusy,
    /// Anything else the browser reported.
    Failed(String),
}

impl MicError {
    /// Map a `DOMException` name from `getUserMedia` / Web Audio.
    pub fn from_dom_name(name: &str, message: &str) -> Self {
        match name {
            "NotAllowedError" | "SecurityError" => Self::PermissionDenied,
            "NotFoundError" | "OverconstrainedError" => Self::NoDevice,
            "NotReadableError" | "AbortError" => Self::DeviceBusy,
            "NotSupportedError" => Self::Unsupported,
            _ => Self::Failed(format!("{name}: {message}")),
        }
    }
}

impl std::fmt::Display for MicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "microphone input is not supported here (needs https or localhost)"),
            Self::PermissionDenied => write!(f, "microphone permission was denied"),
            Self::NoDevice => write!(f, "no microphone found"),
            Self::DeviceBusy => write!(f, "microphone is in use or could not start"),
            Self::Failed(msg) => write!(f, "microphone error: {msg}"),
        }
    }
}

impl std::error::Error for MicError {}
