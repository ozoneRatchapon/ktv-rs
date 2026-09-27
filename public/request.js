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

/** Search text as the booth compares it (src/search/normalize.rs): lower case, letters and digits only, Thai tone
 *  marks (U+0E47–U+0E4D) dropped. `\p{Alphabetic}` keeps Thai vowel signs, as Rust's `is_alphanumeric` does. */
function normalize(text) {
    return text.toLowerCase().replace(/[\u0E47-\u0E4D]/g, '').replace(/[^\p{Alphabetic}\p{N}]/gu, '');
}

/** `/songs.txt` lines (`code \t title \t artist \t aliases`) → songs with a search key per field. */
function parse_songs(text) {
    return text.split('\n').filter(Boolean).map((line) => {
        const [code, title = '', artist = '', aliases = ''] = line.split('\t');
        return { code, title, artist, keys: [normalize(title), normalize(artist), normalize(aliases)] };
    });
}

/** Best matches first, as on the booth: code, title (start, inside), artist, aliases, then every word somewhere. */
function find_songs(songs, query, limit = 30) {
    const needle = normalize(query);
    if (!needle) return [];
    const words = query.split(/\s+/).map(normalize).filter(Boolean);
    const rank = ({ code, keys: [title, artist, aliases] }) => {
        if (code === query.trim()) return 0;
        if (title.startsWith(needle)) return 1;
        if (title.includes(needle)) return 2;
        if (artist.includes(needle)) return 3;
        if (aliases.includes(needle)) return 4;
        if (words.length > 1 && words.every((w) => title.includes(w) || artist.includes(w) || aliases.includes(w))) return 5;
        return -1;
    };
    return songs
        .map((song, i) => ({ song, i, r: rank(song) }))
        .filter(({ r }) => r >= 0)
        .sort((a, b) => a.r - b.r || a.i - b.i)
        .slice(0, limit)
        .map(({ song }) => song);
}

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

    // Song search: the index loads on first use (≈200 KB compressed), so a guest who knows the code never pays for it
    const search = document.getElementById('song_search');
    const results = document.getElementById('results');
    let songs = null;
    const show = () => {
        results.replaceChildren(...find_songs(songs ?? [], search.value).map((song) => {
            const button = document.createElement('button');
            button.type = 'button';
            const code_span = Object.assign(document.createElement('span'), { className: 'code', textContent: song.code });
            const artist_span = Object.assign(document.createElement('span'), { className: 'artist', textContent: song.artist });
            button.append(code_span, song.title, artist_span);
            button.lang = 'th';
            button.addEventListener('click', () => {
                code.value = song.code;
                search.value = song.title;
                results.replaceChildren();
                update();
            });
            const item = document.createElement('li');
            item.append(button);
            return item;
        }));
    };
    search.addEventListener('input', async () => {
        if (!songs) {
            songs = [];
            const response = await fetch('/songs.txt').catch(() => null);
            songs = response?.ok ? parse_songs(await response.text()) : [];
        }
        show();
    });
}

if (typeof module !== 'undefined') {
    module.exports = { USDC_MINT, find_songs, normalize, parse_room, parse_songs, request_link };
} else {
    main();
}
