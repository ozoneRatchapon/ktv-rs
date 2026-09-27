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

## Scope (gated items need an owner go)
- [ ] S1. Tip QR on screen: Solana Pay transfer request (USDC, `reference` key, `memo`) to the singer's wallet or `.sol` name set in Settings. No wallet connection on the booth.
- [ ] S2. Confirm on-chain from the browser: poll `getSignaturesForAddress(reference)` → `getTransaction`, check amount, mint and recipient, dedupe by signature. Update CSP `connect-src` for the RPC. `gated:` choose an RPC (public vs Helius free tier).
- [ ] S3. Tip memo → priority request `[★ TIP]` + on-screen garland toast. The chain acts as the phone-remote relay, so no Durable Object is needed (this partly replaces checklist 33).
- [ ] S4. Night leaderboard payout, optional: the host sends the pot to the best take (built on the party leaderboard).
- [ ] A1. Thai request understanding: lyric snippet / mood / artist in free text → library matches. `gated:` needs a server component (Worker script + LLM key), so pick a provider, budget and privacy-note wording.
- [ ] A2. AI MC: announces tips and requests and gives next-song picks and vocal feedback from the tuning summary.
- [ ] P1. Privacy note + README: wallet addresses and memos are public on-chain.
- [ ] G1. Pilot: one real party or bar night in Bangkok. Count tips, requests and singers. `gated:` owner arranges.
- [ ] G2. Submission: 3-min demo video, pitch deck, the 8 Superteam answers, disclosure of prior work. `gated:` team bios and founder story from the owner.

## Risks to answer up front
- Music licensing: the app plays official YouTube karaoke embeds; tips go to the performer, not for the song. Commercial venue use needs label licences, so pitch parties and streamers first and label partnerships later.
- Thai users rarely hold USDC: onboarding friction (Phantom / Solflare, or a Thai on-ramp) is the main GTM question.
- 14 days: S1–S3 + A1 + G1 + G2 is the minimum winning set; skip S4 and on-chain escrow.
