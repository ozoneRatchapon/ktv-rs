# 004: Search over the full library (8,227 songs)

Date: 2026-09-27 · Host: Apple M5 Pro, `cargo test --release` (native; expect wasm ~2-3x slower) · v0.15.0

Songbook: 43 curated + 8,184 library songs (`assets/library.json`: 920 KB raw, ~300 KB gzip).

| Query | Hits | Normalise per keystroke (before) | Precomputed keys (`Songbook`) |
| --- | --- | --- | --- |
| `รัก` | 937 | 19.0 ms | 0.18 ms |
| `rak` | 417 | 19.9 ms | 0.23 ms |
| `bodyslam` | 58 | 19.5 ms | 0.15 ms |
| `ความรักฉันหายไป` | 1 | 19.8 ms | 1.44 ms |
| `zzzqqq` (no match, retried on the other layout) | 0 | 39.0 ms | 0.33 ms |
| `30500` (code) | 1 | 19.5 ms | 0.17 ms |

One-off after the fetch: parse + index 23 ms. Rendering is capped at 60 cards (+60 per Show more), so a
one-letter query never builds thousands of DOM nodes.
