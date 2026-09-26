# 001 — MPM pitch detector (2026-09-26)

Machine: Apple M5 Pro, macOS 27.0, rustc 1.97.1, Node 24.16.0. Config `MpmConfig::singing` (80–1100 Hz),
frame 2048, hop 1024 (`src/mic/types.rs`). Input: 220 Hz vowel-like tone (2nd harmonic loudest) and silence.

## Reproduce
```sh
cargo bench --bench pitch_detector                                   # native
cargo build --release --no-default-features --target wasm32-wasip1 --bench pitch_detector
node bench/run_wasi.mjs ~/.cargo/target/wasm32-wasip1/release/deps/pitch_detector-*.wasm   # wasm on V8
```

## Results (µs per frame, mean of 2000)
| target | 44.1 kHz vowel | 48 kHz vowel | silence (RMS gate) | CPU share at 48 kHz (46.9 frames/s) |
|---|---|---|---|---|
| native aarch64 | 76.3 | 78.0 | 0.1 | 0.37% |
| wasm32, no SIMD (**shipped**) | 258.8 | 277.0 | 0.5 | 1.30% |
| wasm32 `+simd128` (not shipped) | 95.1 | 101.1 | 0.2 | 0.47% |

## Decisions
- Time-domain NSDF is enough: 0.28 ms per frame on the main thread is ~1.7% of a 16.7 ms display frame. No FFT crate needed.
- `simd128` is 2.7× faster but not enabled: wasm with SIMD fails to load at all on Safari < 16.4 (older iPads are a likely
  KTV device), and the saving is under 1% of one core. Revisit if scoring adds per-frame work.

## Release build size (dx 0.7.10, `dx build --release --platform web`)
| build | wasm | app.js |
|---|---|---|
| before mic/pitch | 846 KB | 57 KB |
| with mic + pitch meter | 878 KB (+32 KB) | 62 KB |
