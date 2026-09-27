// Phone remote wire protocol (roadmap Phase 3), shared by the Worker and its tests. Pure: no Workers APIs.
// Booth and phones talk to one Durable Object per room over WebSockets; every message is small JSON with a `t` tag.
// The booth side lives in src/room (Rust); phones in public/remote.js.

/** 22 base64url chars (132 bits): the first part of SHA-256 of the booth's secret key (see `room_of`). */
export const ROOM_ID = /^[A-Za-z0-9_-]{22}$/;
const SONG_CODE = /^[0-9]{5}$/;
const ROOM_KEY = /^[A-Za-z0-9_-]{43}$/;

export const MAX_PHONES = 32;
/** Booth sockets that have not sent a valid `hello` yet (the real booth sends it as soon as it connects). */
export const MAX_PENDING_BOOTHS = 4;
export const MAX_PHONE_MESSAGE = 256;
export const MAX_BOOTH_MESSAGE = 8192;
export const MAX_REPLY_TEXT = 200;
/** Phone commands: a burst of `BUCKET_SIZE`, then one per `REFILL_MS` (a group queues a few songs, not a flood). */
export const BUCKET_SIZE = 5;
export const REFILL_MS = 10_000;
/** Room state is kept this long after the booth was last heard from, then deleted. */
export const IDLE_MS = 24 * 60 * 60 * 1000;

/** Close codes a phone page explains to the guest. */
export const CLOSE = { replaced: 4001, bad_key: 4003, room_closed: 4004, full: 4029 };

const parse = (text, max) => {
    if (typeof text !== 'string' || text.length > max) return null;
    try {
        const msg = JSON.parse(text);
        return msg && typeof msg === 'object' && !Array.isArray(msg) ? msg : null;
    } catch {
        return null;
    }
};

/** A phone's command, or null: `queue` a 5-digit code, or `skip` / `pause` / `replay` (the booth decides if allowed). */
export function parse_phone(text) {
    const msg = parse(text, MAX_PHONE_MESSAGE);
    switch (msg?.t) {
        case 'queue':
            return typeof msg.code === 'string' && SONG_CODE.test(msg.code) ? { cmd: 'queue', code: msg.code } : null;
        case 'skip':
        case 'pause':
        case 'replay':
            return { cmd: msg.t };
        default:
            return null;
    }
}

/** A booth message, or null: `hello` (proves the key), `state` (relayed to phones), `reply` (to one phone),
 *  `close_room` (the booth made a new link; phones on this one are told to scan again). */
export function parse_booth(text) {
    const msg = parse(text, MAX_BOOTH_MESSAGE);
    switch (msg?.t) {
        case 'hello':
            return typeof msg.key === 'string' && ROOM_KEY.test(msg.key) ? { t: 'hello', key: msg.key } : null;
        case 'state':
            return msg.state && typeof msg.state === 'object' ? { t: 'state', state: msg.state } : null;
        case 'reply':
            return Number.isInteger(msg.to) && typeof msg.ok === 'boolean' && typeof msg.text === 'string'
                ? { t: 'reply', to: msg.to, ok: msg.ok, text: msg.text.slice(0, MAX_REPLY_TEXT) }
                : null;
        case 'close_room':
            return { t: 'close_room' };
        default:
            return null;
    }
}

const base64url = (bytes) => btoa(String.fromCharCode(...bytes)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');

/** The room a booth key opens: base64url(SHA-256(key)), first 22 chars. Only the key's holder can be the booth,
 *  and the Worker stores no key. Must match `room_of` in src/room/link.rs. */
export async function room_of(key) {
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(key));
    return base64url(new Uint8Array(digest)).slice(0, 22);
}

/** Token bucket: `[allowed, next_bucket]`. A new phone starts full. */
export function take_token(bucket, now) {
    const { tokens, at } = bucket ?? { tokens: BUCKET_SIZE, at: now };
    const refilled = Math.min(BUCKET_SIZE, tokens + Math.max(0, now - at) / REFILL_MS);
    return refilled >= 1 ? [true, { tokens: refilled - 1, at: now }] : [false, { tokens: refilled, at: now }];
}
