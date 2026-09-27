use serde::{Deserialize, Serialize};

/// The spoken MC (plan 003 A3): off, or the language it speaks. Off by default: a booth that starts talking
/// on its own surprises people.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McVoice {
    #[default]
    Off,
    Thai,
    English,
}

impl McVoice {
    /// BCP 47 tag for the browser's speech voice; `None` when off.
    pub const fn lang(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::Thai => Some("th-TH"),
            Self::English => Some("en-US"),
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Thai => "ไทย",
            Self::English => "English",
        }
    }

    /// Settings button: Off → Thai → English → Off.
    pub const fn next(self) -> Self {
        match self {
            Self::Off => Self::Thai,
            Self::Thai => Self::English,
            Self::English => Self::Off,
        }
    }
}

/// Something the MC announces. Tips during a song are shown, not spoken: the MC never talks over a singer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McEvent {
    /// A song goes on stage; `tipped` when a guest paid for it (plan 003 S3).
    SongUp { title: String, artist: String, tipped: bool },
    /// The take that just ended, with its tuning score (0–100) and the singer's name if typed.
    TakeEnded { score: u8, singer: Option<String> },
}
