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
- CLS 0.208 comes from **inside the YouTube player iframe**, not our layout. The Lighthouse trace (`--save-assets`)
  has one real shift: score 0.614 in a sub-frame (not the main frame), weighted to 0.208 by frame size; the moving
  node is a 410×231 element (the embed's poster) that collapses to 0×0 when the player takes over. Lighthouse maps it
  onto the main-frame element at that spot (the shortcut list). Main-frame shifts: 0.0001 (nav buttons, Kanit 500).
  Not fixable from our side short of not embedding at load; accepted.
- Best practices 96: third-party cookie issues raised inside the YouTube player frames (not ours).
