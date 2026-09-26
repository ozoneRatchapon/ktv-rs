//! Plain data for pitch detection: detector tuning, raw estimates, and note readings.

/// Tuning for [`super::Mpm`]. Defaults cover the singing range (bass E2 to soprano C6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MpmConfig {
    pub sample_rate: f32,
    /// Lowest pitch reported; sets the longest lag searched (and needs ≥ 2 periods per frame).
    pub min_hz: f32,
    /// Highest pitch reported; sets the shortest lag searched.
    pub max_hz: f32,
    /// A key maximum is chosen once it reaches `cutoff * highest key maximum` (McLeod's k, 0.8–1.0).
    pub cutoff: f32,
    /// Estimates with a lower NSDF peak (periodicity, 0..1) are treated as unvoiced.
    pub min_clarity: f32,
    /// Frames quieter than this RMS (full scale = 1.0) are treated as silence.
    pub min_rms: f32,
}

impl MpmConfig {
    pub fn singing(sample_rate: f32) -> Self {
        Self { sample_rate, min_hz: 80.0, max_hz: 1100.0, cutoff: 0.93, min_clarity: 0.8, min_rms: 0.01 }
    }
}

/// One detected pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchEstimate {
    pub freq_hz: f32,
    /// Height of the chosen NSDF peak: 1.0 is perfectly periodic.
    pub clarity: f32,
}

impl PitchEstimate {
    /// Fractional MIDI note number (A4 = 440 Hz = 69.0).
    pub fn midi(&self) -> f32 {
        69.0 + 12.0 * (self.freq_hz / 440.0).log2()
    }

    pub fn reading(&self) -> NoteReading {
        NoteReading::from_midi(self.midi())
    }
}

/// Nearest equal-tempered note plus how far off it the singer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteReading {
    pub midi: i32,
    /// -50..=50 cents from `midi`.
    pub cents: i32,
}

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

impl NoteReading {
    pub fn from_midi(midi: f32) -> Self {
        let nearest = midi.round();
        Self { midi: nearest as i32, cents: ((midi - nearest) * 100.0).round() as i32 }
    }

    /// Scientific pitch notation, e.g. `A4`, `C#3`.
    pub fn name(&self) -> String {
        let pitch_class = NOTE_NAMES[self.midi.rem_euclid(12) as usize];
        let octave = self.midi.div_euclid(12) - 1;
        format!("{pitch_class}{octave}")
    }
}
