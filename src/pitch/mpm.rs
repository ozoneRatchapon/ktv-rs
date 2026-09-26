//! McLeod Pitch Method (McLeod & Wyvill, "A Smarter Way to Find Pitch", ICMC 2005).
//!
//! Normalised square difference function (NSDF) over the lag range, then the first "key maximum"
//! within `cutoff` of the highest one. Picking the first strong peak rather than the highest
//! avoids octave-down errors; the normalisation keeps it amplitude-independent.
//! Time-domain O(frame × max_lag); buffers are allocated once, `detect` never allocates.

use super::types::{MpmConfig, PitchEstimate};

pub struct Mpm {
    cfg: MpmConfig,
    nsdf: Vec<f32>,
    key_maxima: Vec<usize>,
}

impl Mpm {
    pub fn new(cfg: MpmConfig) -> Self {
        let max_lag = (cfg.sample_rate / cfg.min_hz).ceil() as usize + 1;
        Self { cfg, nsdf: Vec::with_capacity(max_lag), key_maxima: Vec::with_capacity(max_lag / 2) }
    }

    pub fn config(&self) -> &MpmConfig {
        &self.cfg
    }

    /// Pitch of one mono frame (full scale ±1.0), or `None` for silence / unvoiced / out of range.
    /// The frame must span at least two periods of `min_hz` for the lowest notes to be found.
    pub fn detect(&mut self, frame: &[f32]) -> Option<PitchEstimate> {
        let cfg = self.cfg;
        let len = frame.len();
        let min_lag = ((cfg.sample_rate / cfg.max_hz).floor() as usize).max(1);
        let max_lag = ((cfg.sample_rate / cfg.min_hz).ceil() as usize).min(len / 2);
        if max_lag < min_lag + 2 {
            return None;
        }

        let energy = dot(frame, frame);
        if (energy / len as f32).sqrt() < cfg.min_rms {
            return None;
        }

        self.fill_nsdf(frame, energy, max_lag);
        self.find_key_maxima();

        let nsdf = &self.nsdf;
        let highest = self.key_maxima.iter().map(|&lag| nsdf[lag]).fold(f32::MIN, f32::max);
        let chosen = *self.key_maxima.iter().find(|&&lag| nsdf[lag] >= cfg.cutoff * highest)?;

        let (offset, peak) = parabolic_peak(nsdf[chosen - 1], nsdf[chosen], nsdf[chosen + 1]);
        let freq_hz = cfg.sample_rate / (chosen as f32 + offset);
        let clarity = peak.min(1.0);
        match clarity >= cfg.min_clarity && (cfg.min_hz..=cfg.max_hz).contains(&freq_hz) {
            true => Some(PitchEstimate { freq_hz, clarity }),
            false => None,
        }
    }

    /// NSDF n(τ) = 2·r(τ) / m(τ) for τ in 0..=max_lag, with m(τ) updated incrementally.
    fn fill_nsdf(&mut self, frame: &[f32], energy: f32, max_lag: usize) {
        let len = frame.len();
        self.nsdf.clear();
        let mut m = 2.0 * energy;
        for lag in 0..=max_lag {
            if lag > 0 {
                m -= frame[lag - 1] * frame[lag - 1] + frame[len - lag] * frame[len - lag];
            }
            let r = dot(&frame[..len - lag], &frame[lag..]);
            self.nsdf.push(if m > f32::EPSILON { 2.0 * r / m } else { 0.0 });
        }
    }

    /// Highest point of each positive lobe after the first negative-going zero crossing,
    /// keeping only interior peaks (interpolation needs both neighbours). Lags below `min_lag`
    /// are kept too: a tone above `max_hz` must win and be rejected by the range check,
    /// not be reported an octave down from its second period.
    fn find_key_maxima(&mut self) {
        let nsdf = &self.nsdf;
        let len = nsdf.len();
        self.key_maxima.clear();

        let mut pos = nsdf.iter().position(|&v| v <= 0.0).unwrap_or(len);
        while pos < len {
            while pos < len && nsdf[pos] <= 0.0 {
                pos += 1;
            }
            let mut best: Option<usize> = None;
            while pos < len && nsdf[pos] > 0.0 {
                if best.is_none_or(|b| nsdf[pos] > nsdf[b]) {
                    best = Some(pos);
                }
                pos += 1;
            }
            match best {
                Some(lag) if lag + 1 < len => self.key_maxima.push(lag),
                _ => {}
            }
        }
    }
}

/// Vertex of the parabola through three equally spaced samples: (offset from middle, height).
fn parabolic_peak(left: f32, mid: f32, right: f32) -> (f32, f32) {
    let curvature = left - 2.0 * mid + right;
    if curvature.abs() < f32::EPSILON {
        return (0.0, mid);
    }
    let offset = 0.5 * (left - right) / curvature;
    (offset, mid - 0.25 * (left - right) * offset)
}

/// Dot product with independent lanes so LLVM can vectorise (a plain f32 `sum()` is sequential).
fn dot(a: &[f32], b: &[f32]) -> f32 {
    const LANES: usize = 8;
    let mut acc = [0.0f32; LANES];
    let ((a_chunks, a_tail), (b_chunks, b_tail)) = (a.as_chunks::<LANES>(), b.as_chunks::<LANES>());
    let tail: f32 = a_tail.iter().zip(b_tail).map(|(x, y)| x * y).sum();
    for (ca, cb) in a_chunks.iter().zip(b_chunks) {
        for i in 0..LANES {
            acc[i] += ca[i] * cb[i];
        }
    }
    acc.iter().sum::<f32>() + tail
}
