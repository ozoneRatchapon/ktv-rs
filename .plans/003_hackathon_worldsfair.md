# 003 — Colosseum Crypto World's Fair + Superteam Thailand AI × Solana track

Deadline: **2026-10-12** (Colosseum submission; the Superteam listing names no separate date, so treat it as the same).
Enter once on Colosseum with country = Thailand, then also submit on Superteam Earn.
Pre-existing code is allowed if it is disclosed: KTV-RS v0.1–v0.24 (2026-09-26 → 09-28) goes in the submission form.

Targets, most to least realistic: Superteam Thailand AI × Solana ($10k) → Solana ecosystem track (10 × $10k) → Top 20 ($15k) → Grand ($30k).

## Pitch
"พวงมาลัยออนไลน์": Thai crowds already tip singers (money garlands at luk thung shows, tips at karaoke bars).
KTV-RS puts a Solana Pay QR on the TV. Scanning it with a phone tips the singer in USDC; the memo can carry a
song code, which jumps the queue. AI understands Thai requests like "the song that goes …" and runs the room
as a Thai MC.

## Decisions (2026-09-28)
- AI budget is 0 baht and no LLM: A1 is deterministic search, A2 uses reflex locally. Pitch it as "on-device AI that abstains instead of guessing".
- RPC: public devnet.

## Scope (gated items need an owner go)
- [x] S1. Tip QR on screen: Solana Pay transfer request (USDC, `reference` key, `memo`) to the singer's wallet set in Settings. No wallet connection on the booth. Done 2026-09-28 (`src/tip/`, `components/tip_qr.rs`, `tests/tip_test.rs`, e2e decodes the QR with BarcodeDetector). Devnet by default. The on-screen memo is fixed per song (`ktv:<code>`); guests cannot type one in most wallets, so S3 request-by-tip needs a phone page that builds its own link.
  - [ ] `.sol` names: deferred (decided 2026-09-28). SNS names exist on mainnet only, while tips default to devnet; resolving them needs the SNS program derivation or Bonfida's proxy (a third host in CSP). Do it when a mainnet pilot asks for it.
  - [ ] Scan test from a real phone across a room: Phantom and Solflare on devnet.
- [x] S2. Confirm on-chain from the browser: poll `getSignaturesForAddress(reference)` → `getTransaction`, check amount, mint and recipient, dedupe by signature. Update CSP `connect-src` for the RPC. RPC decided: public devnet `api.devnet.solana.com` for the demo ($0); Helius free tier only if rate limits bite. Done 2026-09-28 (`src/tip/rpc.rs` pure parsing, `src/tip/watch.rs` 5 s poll with backoff to 60 s, `tests/tip_confirm_test.rs` on real devnet RPC answers in `tests/fixtures/tip/`, e2e mocks the RPC through CDP `Fetch`). Measured: the public mainnet RPC answers browser origins with 403, so Mainnet needs a keyed Helius URL (Settings → Tip RPC; hosts allowed by `ALLOWED_RPC_HOSTS`, a test keeps CSP in sync). Left for a real phone: a live devnet tip end to end.
- [ ] S3. Tip memo → priority request `[★ TIP]` + on-screen garland toast. The chain acts as the phone-remote relay, so no Durable Object is needed (this partly replaces checklist 33).
- [ ] S4. Night leaderboard payout, optional: the host sends the pot to the best take (built on the party leaderboard).
- [ ] S5. Streamer layout. Works today with no code: TV mode + OBS *Window Capture*; the tip QR shows on stream. A dedicated `?stream` view (vertical 9:16 for TikTok / Facebook Live, no songbook, big QR + tip ticker) only pays off once S2/S3 exist, because an OBS *Browser Source* is its own browser profile and cannot be steered from the booth tab, so the chain (S3) would be its only remote. `gated:` legal: rebroadcasting a label's karaoke video on another platform is outside YouTube's embed terms and draws copyright mutes/strikes. Pitch streamers the tip QR for their own music or licensed tracks, not the GMM library.
- [ ] C1. AI mascot (ComfyUI + MiniMax H3), offline assets only: an original character (ComfyUI image → H3 I2VA, 5–15 s clips) pre-rendered once: cheering, throwing a garland when a tip lands (S3 event), a countdown for the next singer; plus a KTV-RS jingle it sings (original lyrics in `<d>`). Played as a short clip beside the video, never over it. Not an AI singer for library songs: H3 is offline (minutes per clip, not live), and an AI voice singing label lyrics is an unlicensed cover. Best first use is the G2 demo video. `gated:` needs a Windows + NVIDIA GPU (~24 GB) for local ComfyUI (this Mac cannot run the H3 graph) or a fal.ai budget, which conflicts with the 0-baht rule.
- [ ] A1. Thai request search, in the browser (wasm, no server, no model): character-trigram + BM25 over title / artist / Thai romanisation in the library, typo tolerant. Honest scope: the library has no lyric text, so no lyric-snippet search.
- [ ] A2. Tip-memo decisions with reflex (riir-reflex 0.2.3, laya lane, runs on the host laptop, $0, no network): questions `play-next | append-to-end`, `spam?` (noul) and `mood` (choice); act only when `confidence >= threshold`, otherwise take the safe default (append to the queue, no toast). Optional: the app checks `GET http://127.0.0.1:7331/healthz` and works without it.
  - Run it with `RIIR_REFLEX_LAYA=1 RIIR_REFLEX_ALLOWED_ORIGIN=https://ktv-rs.solana-thailand.workers.dev reflex` and send the header `X-Reflex-Lane: laya`; the modelless lane abstains on everything off its demo corpus (measured).
  - Measured 2026-09-28 (M5 Pro, n=3, not a benchmark): English memo → play-next 0.987, not-spam 0.912, celebrating 0.883; scam memo → spam 0.753; Thai memo → wrong answer at confidence 0.046. The HTTP server serves only the english checkpoint, so translate memos to English prompts via fixed templates or gate Thai at the threshold.
  - Before shipping: fit the threshold on ≥16 labelled memos (ρ-quantile, per the skill), add `http://127.0.0.1:7331` to CSP `connect-src`, and check Chrome Private Network Access (an https page calling loopback may need `Access-Control-Allow-Private-Network`).
- [ ] A3. AI MC (template-based, no LLM): announces tips and requests and gives next-song picks and vocal feedback from the tuning summary, using the A2 mood answer.
- [x] P1. Privacy note + README: wallet addresses and memos are public on-chain. Done 2026-09-28.
- [ ] G1. Pilot: one real party or bar night in Bangkok. Count tips, requests and singers. `gated:` owner arranges.
- [ ] G2. Submission: 3-min demo video, pitch deck, the 8 Superteam answers, disclosure of prior work. `gated:` team bios and founder story from the owner.

## Risks to answer up front
- Music licensing: the app plays official YouTube karaoke embeds; tips go to the performer, not for the song. Commercial venue use needs label licences, so pitch parties and streamers first and label partnerships later.
- Thai users rarely hold USDC: onboarding friction (Phantom / Solflare, or a Thai on-ramp) is the main GTM question.
- 14 days: S1–S3 + A1 + G1 + G2 is the minimum winning set; skip S4 and on-chain escrow.
