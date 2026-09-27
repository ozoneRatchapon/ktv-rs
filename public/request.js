// Phone page for plan 003 S3: request a song with a USDC tip. Builds a Solana Pay transfer link to the room's
// wallet with the room's reference and the memo `ktv:req:<code>`; the booth finds it on-chain and queues the song.
// Plain script (no wasm) so it loads fast on a phone. Mints and memo match src/tip (tests/tip_request.test.cjs).
'use strict';

const USDC_MINT = {
    devnet: '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
    mainnet: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
};
const BASE58_KEY = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;
const SONG_CODE = /^[0-9]{5}$/;

/** The booth's `#to=…&c=…&ref=…&room=…` fragment, or null if anything is missing or malformed. */
function parse_room(hash) {
    const params = new URLSearchParams(hash.replace(/^#/, ''));
    const room = { wallet: params.get('to'), cluster: params.get('c'), reference: params.get('ref'), name: params.get('room') ?? '' };
    const valid = BASE58_KEY.test(room.wallet ?? '') && BASE58_KEY.test(room.reference ?? '') && room.cluster in USDC_MINT;
    return valid ? room : null;
}

/** `solana:<wallet>?amount=…&spl-token=…&reference=…&label=…&message=…&memo=ktv:req:<code>` (Solana Pay spec). */
function request_link(room, code, amount) {
    const enc = encodeURIComponent;
    return `solana:${room.wallet}?amount=${amount}&spl-token=${USDC_MINT[room.cluster]}&reference=${room.reference}`
        + `&label=${enc(room.name)}&message=${enc(`Request #${code}`)}&memo=${enc(`ktv:req:${code}`)}`;
}

function main() {
    const room = parse_room(location.hash);
    if (!room) {
        document.getElementById('error').hidden = false;
        return;
    }
    const form = document.getElementById('form');
    const code = document.getElementById('code');
    const pay = document.getElementById('pay');
    const buttons = [...document.querySelectorAll('.amounts button')];
    form.hidden = false;
    document.getElementById('room').textContent = room.name;
    document.getElementById('test_badge').hidden = room.cluster !== 'devnet';
    let amount = '1';
    const update = () => {
        const ready = SONG_CODE.test(code.value);
        pay.href = ready ? request_link(room, code.value, amount) : '#';
        pay.setAttribute('aria-disabled', String(!ready));
    };
    for (const button of buttons) {
        button.addEventListener('click', () => {
            amount = button.dataset.amount;
            for (const b of buttons) b.setAttribute('aria-pressed', String(b === button));
            update();
        });
    }
    code.addEventListener('input', () => {
        code.value = code.value.replace(/\D/g, '').slice(0, 5);
        update();
    });
    form.addEventListener('submit', (e) => e.preventDefault());
    update();
}

if (typeof module !== 'undefined') {
    module.exports = { USDC_MINT, parse_room, request_link };
} else {
    main();
}
