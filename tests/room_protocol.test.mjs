// Unit tests for the phone remote: the Worker's protocol (worker/protocol.js) and the phone page (public/remote.js).
// Run: node --test tests/room_protocol.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import {
    BUCKET_SIZE, MAX_BOOTH_MESSAGE, MAX_PHONE_MESSAGE, MAX_REPLY_TEXT, REFILL_MS, ROOM_ID, parse_booth, parse_phone,
    room_of, take_token,
} from '../worker/protocol.js';

const require = createRequire(import.meta.url);
const remote = require('../public/remote.js');

// Same vector as tests/room_test.rs (Rust `room_of`)
const KEY = 'A'.repeat(43);
const ROOM = 'DwBzhbb51LfusnSGBa_hqY';

test('room id is SHA-256 of the key, as the booth computes it', async () => {
    assert.equal(await room_of(KEY), ROOM);
    assert.match(await room_of('B'.repeat(43)), ROOM_ID);
    assert.notEqual(await room_of('B'.repeat(43)), ROOM);
});

test('phones may send a 5-digit queue request or a playback command, nothing else', () => {
    assert.deepEqual(parse_phone('{"t":"queue","code":"10004"}'), { cmd: 'queue', code: '10004' });
    for (const t of ['skip', 'pause', 'replay', 'volume_up', 'volume_down']) assert.deepEqual(parse_phone(JSON.stringify({ t })), { cmd: t });
    const extra = parse_phone('{"t":"skip","from":1,"cmd":"queue"}');
    assert.deepEqual(extra, { cmd: 'skip' }, 'a phone cannot forge the sender or smuggle fields');
    for (const bad of ['{"t":"queue","code":"1234"}', '{"t":"queue","code":10004}', '{"t":"queue","code":"10004x"}',
        '{"t":"hello","key":"x"}', '{"t":"state","state":{}}', '[]', 'null', 'not json', '']) {
        assert.equal(parse_phone(bad), null, bad);
    }
    assert.equal(parse_phone(JSON.stringify({ t: 'skip', pad: 'x'.repeat(MAX_PHONE_MESSAGE) })), null, 'oversized');
});

test('booth messages: hello needs a key-shaped key, replies are capped, junk is dropped', () => {
    assert.deepEqual(parse_booth(JSON.stringify({ t: 'hello', key: KEY })), { t: 'hello', key: KEY });
    assert.equal(parse_booth(JSON.stringify({ t: 'hello', key: 'short' })), null);
    assert.deepEqual(parse_booth('{"t":"state","state":{"now":null}}'), { t: 'state', state: { now: null } });
    assert.equal(parse_booth('{"t":"state","state":"x"}'), null);
    const reply = parse_booth(JSON.stringify({ t: 'reply', to: 3, ok: true, text: 'x'.repeat(500) }));
    assert.equal(reply.text.length, MAX_REPLY_TEXT);
    assert.equal(parse_booth('{"t":"reply","to":"3","ok":true,"text":"x"}'), null);
    assert.deepEqual(parse_booth('{"t":"close_room"}'), { t: 'close_room' });
    assert.equal(parse_booth(JSON.stringify({ t: 'state', state: { pad: 'x'.repeat(MAX_BOOTH_MESSAGE) } })), null);
});

test('rate limit: a burst, then one command per refill period', () => {
    let bucket = null;
    let allowed = 0;
    for (let i = 0; i < BUCKET_SIZE + 3; i++) {
        const [ok, next] = take_token(bucket, 1000);
        bucket = next;
        if (ok) allowed++;
    }
    assert.equal(allowed, BUCKET_SIZE);
    assert.equal(take_token(bucket, 1000 + REFILL_MS / 2)[0], false);
    const [ok, after] = take_token(bucket, 1000 + REFILL_MS);
    assert.equal(ok, true);
    assert.equal(take_token(after, 1000 + REFILL_MS)[0], false);
    assert.equal(take_token(null, 0)[1].tokens, BUCKET_SIZE - 1, 'a new phone starts full');
});

test('phone page: room from the fragment, socket on the same host', () => {
    assert.equal(remote.parse_room(`#r=${ROOM}`), ROOM);
    assert.equal(remote.parse_room('#r=short'), null);
    assert.equal(remote.parse_room(''), null);
    assert.equal(remote.socket_url({ protocol: 'https:', host: 'ktv.example' }, ROOM), `wss://ktv.example/api/room/${ROOM}?role=phone`);
    assert.equal(remote.socket_url({ protocol: 'http:', host: 'localhost:8788' }, ROOM), `ws://localhost:8788/api/room/${ROOM}?role=phone`);
    assert.equal(remote.next_backoff(1000), 2000);
    assert.equal(remote.next_backoff(20000), 30000);
    assert.ok(remote.CLOSED_FOR_GOOD[4004], 'a replaced link stops reconnecting');
});

test('phone page reads the server messages it shows', () => {
    assert.deepEqual(remote.read_message('{"t":"state","online":true,"state":{"now":null}}'), { state: { now: null }, online: true });
    assert.deepEqual(remote.read_message('{"t":"online","online":false}'), { online: false });
    assert.deepEqual(remote.read_message('{"t":"reply","ok":false,"text":"No song"}'), { reply: { ok: false, text: 'No song' } });
    assert.equal(remote.read_message('{"t":"cmd"}'), null);
    assert.equal(remote.read_message('oops'), null);
});
