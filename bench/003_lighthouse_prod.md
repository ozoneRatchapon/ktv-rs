# 003 — Lighthouse on prod (2026-09-27)

`npx lighthouse@12 https://ktv-rs.solana-thailand.workers.dev/` (default mobile preset: Moto G Power, simulated slow 4G,
4× CPU), headless Chrome on Apple M5 Pro. First visit (empty storage).

| release | Perf | A11y | Best pr. | SEO | FCP | LCP | CLS | Speed idx | TBT |
|---|---|---|---|---|---|---|---|---|---|
| v0.8.0 | 54 | 95 | 96 | 91 | 4.3 s | 4.9 s | 0.347 | 4.3 s | 0 ms |
| v0.9.0 | **88** | **100** | 96 | **100** | **1.4 s** | **2.2 s** | 0.208 | **2.3 s** | 10 ms |

## What changed in v0.9.0
- `main.css` in the static `<head>` (`AssetOptions::css().with_static_head(true)`): it used to be injected by the wasm
  after it ran, so nothing painted until then and everything shifted when styles arrived.
- Boot splash in `index.html` (hidden by CSS once the app mounts into `#main`): first paint no longer waits for the wasm.
- Preloaded fonts (Kanit 400/600/700 Thai + Latin, Outfit Latin) and preconnect to youtube-nocookie.com.
- Favicon 133 KB (6 sizes up to 256 px) → 15 KB (16/32/48) at `/favicon.ico`; real `/robots.txt` (the SPA fallback
  used to answer with HTML).
- WCAG AA contrast: `--text-muted` #64748b → #8a97ab (≥ 4.9:1 on every surface); intro banners solid colours.

## Open
- CLS 0.208, attributed by Lighthouse to the first-visit shortcut list. Not reproducible in a PerformanceObserver trace
  with the same viewport, DPR, slow-4G and 4× CPU emulation (only a 0.000 shift from the nav buttons when Kanit 500
  arrives). Needs a trace/filmstrip from the Lighthouse run itself.
- Best practices 96: third-party cookie issues raised inside the YouTube player frames (not ours).
