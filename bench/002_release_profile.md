# 002 — Release profile: size vs pitch speed (2026-09-27)

Machine: Apple M5 Pro, macOS 27.0, Node 24.16.0, dx 0.7.10. Web size = `tools/build_web.sh` output (after dx's wasm-opt).
Speed = `pitch_detector` bench built for `wasm32-wasip1`, run on V8 (`bench/run_wasi.mjs`), 220 Hz vowel, µs per frame.

## Results
| `[profile.release]` | wasm raw | wasm gzip -9 | 44.1 kHz | 48 kHz | CPU share at 48 kHz |
|---|---|---|---|---|---|
| default (opt 3), before | 1,031,211 | 403,099 | 171.4 | 183.8 | 0.86% |
| opt 3 + `lto` + `codegen-units = 1` (**shipped**) | 882,198 | 339,693 | 171.1 | 182.8 | 0.86% |
| opt "z" + `lto` + `codegen-units = 1` | 882,198 | 339,693 | 386.1 | 412.9 | 1.94% |
| opt "s" + `lto` + `codegen-units = 1` | — | — | 378.1 | 405.3 | 1.90% |

## Decisions
- Ship opt 3 + LTO + one codegen unit: −14% raw / −16% gzip for free.
- "z"/"s" bring no extra size after dx's wasm-opt pass but make pitch detection 2.2× slower. Not worth it.
- Removing the `blake3` dependency (v0.5.0) did not measurably change size; the growth from 846 KB (001) came from new
  features (search, picks, scores, timing tools), not a single dependency.

## Reproduce
```sh
tools/build_web.sh && ls -l dist/assets/*.wasm
cargo build --release --no-default-features --target wasm32-wasip1 --bench pitch_detector
node bench/run_wasi.mjs ~/.cargo/target/wasm32-wasip1/release/deps/pitch_detector-*.wasm
```
