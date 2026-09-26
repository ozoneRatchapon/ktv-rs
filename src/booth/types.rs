use crate::types::QueueItem;

/// Current song + queue. `next_queue_id` keeps queue ids unique across the session (and reloads).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Booth {
    pub current: Option<QueueItem>,
    pub queue: Vec<QueueItem>,
    pub next_queue_id: u64,
}

/// Where a newly requested song goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Replace the current song.
    Now,
    /// Front of the queue (or straight on if nothing is playing).
    Next,
    /// End of the queue (or straight on if nothing is playing).
    Back,
}

/// Who asked for a song; shown in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requester {
    Singer,
    Guest,
    Priority,
    Keypad,
    AddUrl,
    AutoDj,
}

impl Requester {
    pub fn label(self) -> &'static str {
        match self {
            Self::Singer => "Singer",
            Self::Guest => "Guest",
            Self::Priority => "Priority",
            Self::Keypad => "Remote Code",
            Self::AddUrl => "YouTube Direct",
            Self::AutoDj => "Smart Auto-DJ",
        }
    }
}
