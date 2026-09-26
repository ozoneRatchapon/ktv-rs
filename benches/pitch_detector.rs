//! `cargo bench --bench pitch_detector` — µs per frame for the singing config.
//! Record results as `bench/NNN_<name>.md` (next free number).

use std::f32::consts::TAU;
use std::hint::black_box;
use std::time::Instant;

use app::pitch::{Mpm, MpmConfig};

const FRAME: usize = 2048;
const ITERS: u32 = 2_000;

fn vowel(sample_rate: f32, freq: f32) -> Vec<f32> {
    (0..FRAME)
        .map(|i| {
            let t = i as f32 / sample_rate;
            [(1.0, 0.6), (2.0, 1.0), (3.0, 0.5), (4.0, 0.3)].iter().map(|&(k, a): &(f32, f32)| 0.2 * a * (TAU * freq * k * t).sin()).sum()
        })
        .collect()
}

fn main() {
    println!("| sample rate | input | µs/frame (mean of {ITERS}) | frames/s at hop 1024 | CPU share |");
    println!("|---|---|---|---|---|");
    for sample_rate in [44_100.0f32, 48_000.0] {
        let mut mpm = Mpm::new(MpmConfig::singing(sample_rate));
        for (label, frame) in [("vowel 220 Hz", vowel(sample_rate, 220.0)), ("silence (RMS gate)", vec![0.0; FRAME])] {
            for _ in 0..50 {
                black_box(mpm.detect(black_box(&frame)));
            }
            let start = Instant::now();
            for _ in 0..ITERS {
                black_box(mpm.detect(black_box(&frame)));
            }
            let us = start.elapsed().as_secs_f64() * 1e6 / f64::from(ITERS);
            let fps = f64::from(sample_rate) / 1024.0;
            let share = us * fps / 1e4;
            println!("| {sample_rate} | {label} | {us:.1} | {fps:.1} | {share:.2}% |");
        }
    }
}
