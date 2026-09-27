# 005 — Lighthouse on prod after the full library (2026-09-27)

Same method as [003](003_lighthouse_prod.md): `npx lighthouse@12` (12.8.2) on https://ktv-rs.solana-thailand.workers.dev/,
default mobile preset (Moto G Power, simulated slow 4G, 4× CPU) ×3 plus `--preset=desktop` ×1, headless Chrome on
Apple M5 Pro, first visit. Lighthouse runs without an autoplay flag, so the page loads with autoplay blocked.

| release | run | Perf | A11y | Best pr. | SEO | FCP | LCP | TBT | CLS | Longest task |
|---|---|---|---|---|---|---|---|---|---|---|
| v0.17.2 | mobile ×3 | 86-89 | 96 | 96 | 100 | 1.0-1.5 s | 1.4-2.4 s | 120-150 ms | 0.194 | 264 ms |
| v0.17.2 | desktop | 86 | 96 | 96 | 100 | 0.5 s | 1.3 s | 20 ms | 0.227 | — |
| v0.17.3 | mobile ×3 | **88-89** | **100** | 96 | 100 | 1.5 s | 2.1-2.2 s | **10-20 ms** | 0.194 | **71-75 ms** |
| v0.17.3 | desktop | 84 | **100** | 96 | 100 | 0.5 s | 1.3 s | **0 ms** | 0.227 | none |

## What changed in v0.17.3
- **Long task (TBT):** the 264 ms task was installing the library. Split natively: JSON parse ~1.5 ms, search keys
  ~25 ms (93%). `search_key` now fills one buffer and takes fast paths (ASCII without Unicode tables; the Thai block
  from a 128-entry table built once from the same `is_alphanumeric` rule): **25 ms → 0.8 ms** for 8,189 songs,
  identical output (`test_search_key_matches_reference_on_every_song` checks every song against the plain definition).
- **A11y 96 → 100:** the paused-state Play button (amber, now shown whenever autoplay is blocked) had white text at
  3.18:1; now dark text (~6:1).

## Open
- CLS 0.194 / 0.227 is still the YouTube embed's poster collapsing inside its iframe (see 003); not ours.
- Best practices 96: third-party cookie notices inside the YouTube frames.
- Perf score varies ±3 between identical runs; compare ranges, not single runs.
