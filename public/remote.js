// Phone remote (roadmap Phase 3): a guest scans the booth's QR, sees what is on and next, and queues songs; skip /
// pause / replay work only if the host allows them. Talks to the room's Durable Object (worker/room.js) over a
// WebSocket; the room id rides in the `#r=` fragment. Plain script (no wasm) so it loads fast on a phone.
'use strict';

const { attach_search } = typeof module !== 'undefined' ? require('./song_search.js') : KtvSongSearch;

const ROOM_ID = /^[A-Za-z0-9_-]{22}$/;
const SONG_CODE = /^[0-9]{5}$/;
/** worker/protocol.js `CLOSE`: codes after which reconnecting cannot help. */
const CLOSED_FOR_GOOD = { 4004: 'The booth made a new code. Scan the QR on the booth again.' };
const MIN_BACKOFF_MS = 1000;
const MAX_BACKOFF_MS = 30000;

/** The room from `#r=<room>`, or null. */
function parse_room(hash) {
    const room = new URLSearchParams(hash.replace(/^#/, '')).get('r');
    return ROOM_ID.test(room ?? '') ? room : null;
}

/** `wss://host/api/room/<room>?role=phone` (`ws://` on a local http page). */
function socket_url(location, room) {
    const scheme = location.protocol === 'https:' ? 'wss' : 'ws';
    return `${scheme}://${location.host}/api/room/${room}?role=phone`;
}

/** Next reconnect wait: doubles up to the cap. */
function next_backoff(ms) {
    return Math.min(ms * 2, MAX_BACKOFF_MS);
}

/** What the page shows for a server message: `{ state, online }`, `{ online }` or `{ reply }`; null if unknown. */
function read_message(text) {
    let msg;
    try {
        msg = JSON.parse(text);
    } catch {
        return null;
    }
    switch (msg?.t) {
        case 'state':
            return { state: msg.state ?? null, online: msg.online === true };
        case 'online':
            return { online: msg.online === true };
        case 'reply':
            return { reply: { ok: msg.ok === true, text: String(msg.text ?? '') } };
        default:
            return null;
    }
}

/** The booth's saved medleys in a state message (titles and part counts), keeping only well-formed entries. */
function state_medleys(state) {
    const list = Array.isArray(state?.medleys) ? state.medleys : [];
    return list.filter((m) => typeof m?.title === 'string' && m.title !== '' && Number.isInteger(m.parts));
}

function main() {
    const room = parse_room(location.hash);
    const $ = (id) => document.getElementById(id);
    if (!room) {
        $('error').hidden = false;
        return;
    }
    $('app').hidden = false;
    const code = $('code');
    const queue = $('queue');
    const status = $('status');
    const reply = $('reply');
    let ws = null;
    let online = false;
    let playback = false;
    let backoff = MIN_BACKOFF_MS;
    let retry = null;
    let closed_for_good = false;

    const song_line = (song) => {
        const item = document.createElement('li');
        const code_span = Object.assign(document.createElement('span'), { className: 'code', textContent: song.code });
        const artist_span = Object.assign(document.createElement('span'), { className: 'artist', textContent: song.artist });
        item.lang = 'th';
        item.append(code_span, song.title, artist_span);
        if (song.requester === '★ TIP' || song.requester === '📱 Phone') {
            item.append(Object.assign(document.createElement('span'), { className: 'tag', textContent: song.requester }));
        }
        return item;
    };
    const medley_line = (medley) => {
        const item = document.createElement('li');
        const title = Object.assign(document.createElement('span'), { className: 'title', textContent: medley.title });
        title.append(Object.assign(document.createElement('span'), { className: 'artist', textContent: ` · ${medley.parts} parts` }));
        const button = Object.assign(document.createElement('button'), { type: 'button', textContent: 'Queue' });
        button.setAttribute('aria-label', `Queue the medley ${medley.title}`);
        button.addEventListener('click', () => send({ t: 'queue_medley', title: medley.title }));
        item.lang = 'th';
        item.append(title, button);
        return item;
    };
    const render_state = (state) => {
        $('room').textContent = state?.room ?? '';
        const now = state?.now;
        $('now_title').textContent = now ? now.title : 'Nothing on stage';
        $('now_artist').textContent = now ? now.artist : '';
        const next = state?.next ?? [];
        $('next').replaceChildren(...next.map(song_line));
        const more = (state?.waiting ?? 0) - next.length;
        $('more').textContent = more > 0 ? `+${more} more waiting` : next.length ? '' : 'The queue is empty';
        playback = state?.playback === true;
        $('volume').textContent = Number.isInteger(state?.volume) ? `Vol ${state.volume}` : '';
        const medleys = state_medleys(state);
        $('medleys').replaceChildren(...medleys.map(medley_line));
        $('medleys_section').hidden = medleys.length === 0;
    };
    const render = () => {
        const open = ws?.readyState === WebSocket.OPEN;
        status.textContent = closed_for_good || (online ? 'Connected to the booth' : open ? 'The booth is offline' : 'Connecting…');
        status.dataset.online = String(online);
        $('controls').hidden = !(online && playback);
        const ready = online && SONG_CODE.test(code.value);
        queue.disabled = !ready;
        for (const button of $('medleys').querySelectorAll('button')) button.disabled = !online;
    };
    const show_reply = ({ ok, text }) => {
        reply.textContent = text;
        reply.dataset.ok = String(ok);
    };
    const send = (msg) => {
        if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg));
    };

    const connect = () => {
        clearTimeout(retry);
        if (closed_for_good || document.hidden) return;
        const socket = new WebSocket(socket_url(location, room));
        ws = socket;
        socket.addEventListener('open', () => {
            backoff = MIN_BACKOFF_MS;
            render();
        });
        socket.addEventListener('message', (event) => {
            const msg = read_message(event.data);
            if (!msg) return;
            if ('state' in msg) render_state(msg.state);
            if ('online' in msg) online = msg.online;
            if (msg.reply) show_reply(msg.reply);
            render();
        });
        socket.addEventListener('close', (event) => {
            if (ws !== socket) return;
            ws = null;
            online = false;
            closed_for_good = CLOSED_FOR_GOOD[event.code] ?? false;
            render();
            if (!closed_for_good && !document.hidden) {
                retry = setTimeout(connect, backoff);
                backoff = next_backoff(backoff);
            }
        });
        render();
    };
    // A phone in a pocket holds no connection: close when hidden, reconnect when shown again
    document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
            const socket = ws;
            ws = null;
            socket?.close(1000);
        } else if (!ws) {
            backoff = MIN_BACKOFF_MS;
            connect();
        }
    });

    code.addEventListener('input', () => {
        code.value = code.value.replace(/\D/g, '').slice(0, 5);
        render();
    });
    $('form').addEventListener('submit', (e) => {
        e.preventDefault();
        if (!SONG_CODE.test(code.value)) return;
        send({ t: 'queue', code: code.value });
        code.value = '';
        $('song_search').value = '';
        render();
    });
    for (const button of document.querySelectorAll('#controls button')) {
        button.addEventListener('click', () => send({ t: button.dataset.cmd }));
    }
    attach_search($('song_search'), $('results'), (song) => {
        code.value = song.code;
        render();
        queue.focus();
    });
    connect();
}

if (typeof module !== 'undefined') {
    module.exports = { CLOSED_FOR_GOOD, next_backoff, parse_room, read_message, socket_url, state_medleys };
} else {
    main();
}
