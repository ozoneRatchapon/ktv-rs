use std::f32::consts::TAU;

use app::pitch::{Mpm, MpmConfig, NoteReading, PitchEstimate};

const FRAME: usize = 2048;

/// Sum of harmonics `(multiple, amplitude)` of `freq`, peak-normalised to `peak`.
fn tone(sample_rate: f32, freq: f32, harmonics: &[(f32, f32)], peak: f32) -> Vec<f32> {
    let raw: Vec<f32> = (0..FRAME)
        .map(|i| {
            let t = i as f32 / sample_rate;
            harmonics.iter().map(|&(k, a)| a * (TAU * freq * k * t).sin()).sum()
        })
        .collect();
    let max = raw.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    raw.into_iter().map(|v| v * peak / max).collect()
}

fn sine(sample_rate: f32, freq: f32) -> Vec<f32> {
    tone(sample_rate, freq, &[(1.0, 1.0)], 0.5)
}

fn assert_close(found: Option<PitchEstimate>, expected_hz: f32) {
    let est = found.unwrap_or_else(|| panic!("no pitch found for {expected_hz} Hz"));
    let cents = 1200.0 * (est.freq_hz / expected_hz).log2();
    assert!(cents.abs() < 5.0, "{expected_hz} Hz detected as {} Hz ({cents:.1} cents)", est.freq_hz);
    assert!(est.clarity > 0.9, "{expected_hz} Hz: clarity {}", est.clarity);
}

#[test]
fn test_sine_across_singing_range_at_both_common_rates() {
    for sample_rate in [44_100.0, 48_000.0] {
        let mut mpm = Mpm::new(MpmConfig::singing(sample_rate));
        for freq in [82.41, 110.0, 196.0, 261.63, 440.0, 659.26, 1046.5] {
            assert_close(mpm.detect(&sine(sample_rate, freq)), freq);
        }
    }
}

#[test]
fn test_voice_like_harmonics_do_not_cause_octave_errors() {
    let mut mpm = Mpm::new(MpmConfig::singing(48_000.0));
    // Second harmonic louder than the fundamental, as in many sung vowels
    let vowel = [(1.0, 0.6), (2.0, 1.0), (3.0, 0.5), (4.0, 0.3), (5.0, 0.2)];
    for freq in [130.81, 220.0, 392.0] {
        assert_close(mpm.detect(&tone(48_000.0, freq, &vowel, 0.5)), freq);
    }
}

#[test]
fn test_amplitude_independent_above_the_silence_gate() {
    let mut mpm = Mpm::new(MpmConfig::singing(48_000.0));
    for peak in [0.03, 0.3, 1.0] {
        assert_close(mpm.detect(&tone(48_000.0, 220.0, &[(1.0, 1.0)], peak)), 220.0);
    }
}

#[test]
fn test_silence_and_noise_are_unvoiced() {
    let mut mpm = Mpm::new(MpmConfig::singing(48_000.0));
    assert_eq!(mpm.detect(&[0.0; FRAME]), None);
    assert_eq!(mpm.detect(&tone(48_000.0, 220.0, &[(1.0, 1.0)], 0.005)), None, "below the RMS gate");

    // Deterministic white noise (xorshift32)
    let mut state = 0x9E37_79B9u32;
    let noise: Vec<f32> = (0..FRAME)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state as f32 / u32::MAX as f32 - 0.5) * 0.8
        })
        .collect();
    assert_eq!(mpm.detect(&noise), None);
}

#[test]
fn test_out_of_range_and_short_frames_are_rejected() {
    let mut mpm = Mpm::new(MpmConfig::singing(48_000.0));
    assert_eq!(mpm.detect(&sine(48_000.0, 40.0)), None, "below min_hz");
    // Above max_hz must be rejected, not reported an octave down (second-period peak)
    let bright = [(1.0, 1.0), (2.0, 0.5), (3.0, 0.33)];
    for freq in [1500.0, 1800.0] {
        assert_eq!(mpm.detect(&tone(48_000.0, freq, &bright, 0.5)), None, "{freq} Hz is above max_hz");
    }
    assert_eq!(mpm.detect(&sine(48_000.0, 220.0)[..64]), None, "frame too short for the lag range");
}

#[test]
fn test_note_readings() {
    let reading = |freq_hz| PitchEstimate { freq_hz, clarity: 1.0 }.reading();
    assert_eq!(reading(440.0), NoteReading { midi: 69, cents: 0 });
    assert_eq!(reading(440.0).name(), "A4");
    assert_eq!(reading(261.63).name(), "C4");
    assert_eq!(reading(466.16).name(), "A#4");
    assert_eq!(reading(82.41).name(), "E2");
    assert_eq!(reading(445.0), NoteReading { midi: 69, cents: 20 });
    assert_eq!(reading(435.0), NoteReading { midi: 69, cents: -20 });
    assert_eq!(NoteReading::from_midi(0.0).name(), "C-1");
}
