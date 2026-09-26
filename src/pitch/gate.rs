//! Room check + noise gate for the singer's mic: frames not clearly louder than the room are silence.
//!
//! The first [`ROOM_CHECK_FRAMES`] frames after the mic opens measure the room (median RMS, so a cough
//! does not count); the gate then stays fixed for the session. A floor that kept adapting would creep
//! up to the level of a singer who never pauses and shut them out. Toggling the mic re-checks.

/// Frames measured before the gate opens (~1 s at 48 kHz with a 1024-sample hop).
pub const ROOM_CHECK_FRAMES: usize = 48;
/// A frame must be this many times the room's RMS (~6 dB) to count as singing.
pub const GATE_RATIO: f32 = 2.0;
/// Gate never below this RMS (a silent room still has converter noise).
pub const MIN_GATE_RMS: f32 = 0.01;
/// Gate never above this RMS, so a very loud room (or singing through the check) cannot mute the mic.
pub const MAX_GATE_RMS: f32 = 0.1;

/// Root mean square of a frame (full scale ±1.0).
pub fn rms(frame: &[f32]) -> f32 {
    match frame.len() {
        0 => 0.0,
        len => (frame.iter().map(|s| s * s).sum::<f32>() / len as f32).sqrt(),
    }
}

#[derive(Debug, Clone)]
pub enum NoiseGate {
    /// Still measuring the room: RMS of each frame so far.
    Checking(Vec<f32>),
    /// Frames at or above this RMS pass.
    Open { threshold: f32 },
}

impl Default for NoiseGate {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseGate {
    pub fn new() -> Self {
        Self::Checking(Vec::with_capacity(ROOM_CHECK_FRAMES))
    }

    /// Feed one frame's RMS; `true` if the frame should be analysed as singing.
    pub fn pass(&mut self, level: f32) -> bool {
        match self {
            Self::Open { threshold } => level >= *threshold,
            Self::Checking(levels) => {
                levels.push(level);
                if levels.len() >= ROOM_CHECK_FRAMES {
                    *self = Self::Open { threshold: threshold_for(levels) };
                }
                false
            }
        }
    }

    pub fn is_checking(&self) -> bool {
        matches!(self, Self::Checking(_))
    }

    /// The gate level once the room check is done.
    pub fn threshold(&self) -> Option<f32> {
        match self {
            Self::Open { threshold } => Some(*threshold),
            Self::Checking(_) => None,
        }
    }
}

fn threshold_for(levels: &mut [f32]) -> f32 {
    levels.sort_by(f32::total_cmp);
    let room = levels[levels.len() / 2];
    (room * GATE_RATIO).clamp(MIN_GATE_RMS, MAX_GATE_RMS)
}
