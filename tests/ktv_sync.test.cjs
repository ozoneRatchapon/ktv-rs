// Unit tests for the player sync core (assets/ktv_sync.js). Run: node --test tests/ktv_sync.test.cjs
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const core = require('../assets/ktv_sync.js');

const { FRAME, LEAD, ACTION, YT_STATE } = core;

// Headless env: manual clock, recorded postMessage traffic and Rust messages, in-memory storage
function fake_env(stored = {}) {
    const env = {
        clock: 0,
        posts: [],
        sent: [],
        store: Object.assign({}, stored),
        frames: new Set([FRAME.KARAOKE, FRAME.GUIDE]),
        now: () => env.clock,
        post: (frame, msg) => env.posts.push({ frame, msg: JSON.parse(msg) }),
        has_frame: (frame) => env.frames.has(frame),
        send: (msg) => env.sent.push(msg),
        storage: { get: (k) => (k in env.store ? env.store[k] : null), set: (k, v) => { env.store[k] = v; } },
        advance: (ms) => { env.clock += ms; },
        commands: (frame, func) => env.posts.filter((p) => p.frame === frame && p.msg.func === func),
        clear: () => { env.posts.length = 0; env.sent.length = 0; },
    };
    return env;
}

// Karaoke reports its time; guide reports time + state (as YouTube infoDelivery does)
function report(sync, karaoke_sec, guide_sec, guide_state = YT_STATE.PLAYING) {
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: karaoke_sec } });
    if (guide_sec !== undefined) {
        sync.on_message(true, { event: 'infoDelivery', info: { currentTime: guide_sec, playerState: guide_state } });
    }
}

test('next_rate: bands and hysteresis', () => {
    assert.equal(core.next_rate(0.3, 1), 1.1);
    assert.equal(core.next_rate(-0.3, 1), 0.9);
    assert.equal(core.next_rate(0.1, 1), 1.05);
    assert.equal(core.next_rate(-0.1, 1), 0.95);
    assert.equal(core.next_rate(0.02, 1.05), 1);
    // 0.03..0.08: keep whatever rate is running
    assert.equal(core.next_rate(0.05, 1.05), 1.05);
    assert.equal(core.next_rate(-0.05, 0.95), 0.95);
});

test('learn_lead: adds error, clamps to [0, 1.5], ignores failed seeks', () => {
    assert.equal(core.learn_lead(0.1, -0.02), 0.1 - 0.02);
    assert.equal(core.learn_lead(0.1, -0.5), 0);
    assert.equal(core.learn_lead(1.4, 0.9), 1.5);
    assert.equal(core.learn_lead(0.3, 1.0), 0.3);
    assert.equal(core.learn_lead(0.3, -2), 0.3);
    assert.equal(core.learn_lead(0.3, NaN), 0.3);
});

test('parse_lead: accepts only numbers within range', () => {
    assert.equal(core.parse_lead('0.077'), 0.077);
    assert.equal(core.parse_lead('0'), 0);
    assert.equal(core.parse_lead('1.5'), 1.5);
    for (const raw of [null, undefined, '', 'abc', '-0.1', '1.6', 'NaN']) {
        assert.equal(core.parse_lead(raw), null, String(raw));
    }
});

test('decide_guide: precedence of inactive, hold, release, settle, reseek, rate', () => {
    const base = { active: true, paused: false, target: 10, guide_now: 10, waiting: false, settling: false, rate_now: 1 };
    assert.equal(core.decide_guide({ ...base, active: false }).kind, ACTION.NONE);
    assert.equal(core.decide_guide({ ...base, paused: true }).kind, ACTION.NONE);
    assert.equal(core.decide_guide({ ...base, target: -1, waiting: true }).kind, ACTION.HOLD);
    assert.deepEqual(core.decide_guide({ ...base, waiting: true, settling: true }), { kind: ACTION.RELEASE, target: 10 });
    assert.equal(core.decide_guide({ ...base, settling: true, guide_now: 50 }).kind, ACTION.NONE);
    assert.equal(core.decide_guide({ ...base, guide_now: null }).kind, ACTION.NONE);
    assert.deepEqual(core.decide_guide({ ...base, guide_now: 8.5 }), { kind: ACTION.RESEEK, target: 10, err: 1.5 });
    assert.deepEqual(core.decide_guide({ ...base, guide_now: 10.1 }), { kind: ACTION.RATE, err: 10 - 10.1, rate: 0.95 });
});

test('leads load from storage, falling back on missing or invalid values', () => {
    const env = fake_env({ [LEAD.COLD.key]: '0.077', [LEAD.PAIR.key]: 'garbage' });
    const leads = core.create_sync(env).debug().leads;
    assert.deepEqual(leads, { RESEEK: LEAD.RESEEK.fallback, COLD: 0.077, PAIR: LEAD.PAIR.fallback });
});

test('intro hold: guide pauses while target < 0, then cold-starts with the COLD lead', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(-5, 1);
    sync.set_start(0);
    report(sync, 2, 0);
    sync.switch_vocal(true);
    env.clear();

    assert.equal(sync.tick().kind, ACTION.HOLD);
    assert.equal(env.commands(FRAME.GUIDE, 'pauseVideo').length, 1);

    report(sync, 6, 0, YT_STATE.PAUSED);
    env.clear();
    assert.equal(sync.tick().kind, ACTION.RELEASE);
    const [seek] = env.commands(FRAME.GUIDE, 'seekTo');
    assert.equal(seek.msg.args[0], 1 + LEAD.COLD.fallback);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 1);
});

test('first error after a seek trains that seek kind and persists it', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.set_start(30);
    report(sync, 30, 0);
    sync.switch_vocal(true); // playing -> COLD start at 30 + 0.1

    // Still settling: time reports are ignored
    report(sync, 30.5, 30.5);
    assert.equal(sync.tick().kind, ACTION.NONE);

    env.advance(1600);
    // Guide landed 0.03s ahead of karaoke: lead shrinks by 0.03
    report(sync, 40, 40.03);
    const action = sync.tick();
    assert.equal(action.kind, ACTION.RATE);
    const learned = sync.debug().leads.COLD;
    assert.ok(Math.abs(learned - (LEAD.COLD.fallback - 0.03)) < 1e-9, String(learned));
    assert.equal(env.store[LEAD.COLD.key], String(learned));

    // Only the first error after a seek trains
    report(sync, 41, 41.05);
    sync.tick();
    assert.equal(sync.debug().leads.COLD, learned);
});

test('large drift re-seeks the guide without restarting it and without training', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.switch_vocal(true);
    env.advance(2000);
    report(sync, 20, 20); // consumes the COLD learning
    sync.tick();
    env.clear();

    report(sync, 25, 22);
    assert.equal(sync.tick().kind, ACTION.RESEEK);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo')[0].msg.args[0], 25 + LEAD.RESEEK.fallback);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 0);
});

test('scrub while paused seeks both players with the PAIR lead but keeps the guide paused', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(3, 1);
    sync.switch_vocal(true);
    sync.set_paused(true);
    env.clear();

    sync.seek_all(60);
    assert.deepEqual(env.commands(FRAME.KARAOKE, 'seekTo')[0].msg.args, [60, true]);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo')[0].msg.args[0], 63 + LEAD.PAIR.fallback);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 0);
    assert.equal(sync.tick().kind, ACTION.NONE);
    assert.equal(sync.debug().karaoke_time, 60);
});

test('pause freezes karaoke time; resume restarts the guide with the PAIR lead', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    report(sync, 10, 0);
    sync.switch_vocal(true);
    env.advance(500);
    sync.set_paused(true);
    env.advance(5000);
    assert.equal(sync.debug().karaoke_time, 10.5);
    assert.deepEqual(env.sent.slice(-1), ['PAUSE_STATE:1']);
    env.clear();

    sync.set_paused(false);
    assert.equal(env.commands(FRAME.KARAOKE, 'playVideo').length, 1);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo')[0].msg.args[0], 10.5 + LEAD.PAIR.fallback);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 1);
    assert.deepEqual(env.sent, ['PAUSE_STATE:0']);
});

test('pause is a no-op without a karaoke player (standby)', () => {
    const env = fake_env();
    env.frames.clear();
    const sync = core.create_sync(env);
    sync.toggle_playback();
    assert.equal(sync.debug().paused, false);
    assert.deepEqual(env.sent, []);
});

test('guide messages never drive karaoke time or end-of-song', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(12);
    sync.on_message(true, { event: 'infoDelivery', info: { currentTime: 99 } });
    sync.on_message(true, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.equal(sync.debug().karaoke_time, 12);
    assert.deepEqual(env.sent, []);

    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.deepEqual(env.sent, ['ended']);
});

test('guide error falls back to karaoke audio and reports the code', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.switch_vocal(true);
    env.clear();

    sync.on_message(true, { event: 'onError', info: 150 });
    assert.equal(sync.debug().original, false);
    assert.equal(env.commands(FRAME.KARAOKE, 'unMute').length, 1);
    assert.equal(env.commands(FRAME.GUIDE, 'pauseVideo').length, 1);
    assert.deepEqual(env.sent, ['GUIDE_ERROR:150']);
});

test('progress reports floored karaoke time and re-asserts karaoke mute while original vocal is on', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(42);
    env.advance(900);
    sync.switch_vocal(true);
    env.clear();
    // 'listening' handshakes are not commands; parse them too
    sync.progress();
    assert.deepEqual(env.sent, ['TIME:42']);
    assert.equal(env.posts.filter((p) => p.msg.event === 'listening').length, 2);
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 1);
});

test('load_song resets vocal to karaoke', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.switch_vocal(true);
    sync.load_song(7, 1.01);
    assert.equal(sync.debug().original, false);
    assert.equal(sync.tick().kind, ACTION.NONE);
});

test('is_warm: only players that have played count as warm', () => {
    for (const s of [YT_STATE.PLAYING, YT_STATE.PAUSED, YT_STATE.BUFFERING]) assert.equal(core.is_warm(s), true, String(s));
    for (const s of [undefined, YT_STATE.UNSTARTED, YT_STATE.CUED, YT_STATE.ENDED]) assert.equal(core.is_warm(s), false, String(s));
});

test('cold first play (guide cued) does not train the COLD lead; the next warm start does', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    report(sync, 30, 0, YT_STATE.CUED);
    sync.switch_vocal(true);

    // Unbuffered guide lands 0.26s late: corrected by rate, lead untouched
    env.advance(1600);
    report(sync, 40, 39.74);
    assert.equal(sync.tick().kind, ACTION.RATE);
    assert.equal(sync.debug().leads.COLD, LEAD.COLD.fallback);
    assert.equal(LEAD.COLD.key in env.store, false);

    // Toggle off/on: the guide is now warm (paused), so its landing error trains COLD
    sync.switch_vocal(false);
    report(sync, 41, 41, YT_STATE.PAUSED);
    sync.switch_vocal(true);
    env.advance(1600);
    report(sync, 43, 43.02);
    sync.tick();
    assert.ok(Math.abs(sync.debug().leads.COLD - (LEAD.COLD.fallback - 0.02)) < 1e-9);
});

test('load_song forgets the previous guide state, so the new guide starts cold', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    report(sync, 10, 10);
    sync.load_song(0, 1);
    sync.switch_vocal(true);
    env.advance(1600);
    report(sync, 20, 19.8);
    sync.tick();
    assert.equal(sync.debug().leads.COLD, LEAD.COLD.fallback);
});

test('load_song clears a pause carried over from the previous song and tells Rust', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_paused(true);
    env.clear();
    sync.load_song(0, 1);
    assert.equal(sync.debug().paused, false);
    assert.deepEqual(env.sent, ['PAUSE_STATE:0']);

    env.clear();
    sync.load_song(0, 1);
    assert.deepEqual(env.sent, []);
});

test('restart seeks both players to the start and resumes a paused song', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(2, 1);
    report(sync, 40, 42);
    sync.switch_vocal(true);
    sync.set_paused(true);
    env.clear();

    sync.restart(18);
    assert.equal(sync.debug().paused, false);
    assert.equal(sync.debug().karaoke_time, 18);
    assert.equal(env.commands(FRAME.KARAOKE, 'playVideo').length, 1);
    assert.deepEqual(env.commands(FRAME.KARAOKE, 'seekTo').at(-1).msg.args, [18, true]);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo').at(-1).msg.args[0], 20 + LEAD.PAIR.fallback);
    assert.deepEqual(env.sent, ['PAUSE_STATE:0']);

    // Playing: no pause-state traffic, just the seek
    env.clear();
    sync.restart(0);
    assert.deepEqual(env.sent, []);
    assert.equal(env.commands(FRAME.KARAOKE, 'playVideo').length, 0);
    assert.deepEqual(env.commands(FRAME.KARAOKE, 'seekTo')[0].msg.args, [0, true]);
});

test('pause and play from the karaoke player itself are mirrored: guide follows, no command echoed back', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.set_start(10);
    sync.switch_vocal(true);
    report(sync, 20, 20);
    env.clear();

    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PAUSED });
    assert.deepEqual(env.sent, ['PAUSE_STATE:1']);
    assert.equal(env.commands(FRAME.KARAOKE, 'pauseVideo').length, 0);
    assert.equal(env.commands(FRAME.GUIDE, 'pauseVideo').length, 1);
    const frozen = sync.debug().karaoke_time;
    env.advance(5000);
    assert.equal(sync.debug().karaoke_time, frozen);

    env.clear();
    sync.on_message(false, { event: 'infoDelivery', info: { playerState: YT_STATE.PLAYING } });
    assert.deepEqual(env.sent, ['PAUSE_STATE:0']);
    assert.equal(env.commands(FRAME.KARAOKE, 'playVideo').length, 0);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 1);
});

test('karaoke state reports right after our own play/pause command are not mirrored', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.set_start(0);
    sync.set_paused(false);
    // A stale PAUSED report still in flight when our playVideo lands must not re-pause the song
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PAUSED });
    assert.equal(sync.debug().paused, false);
    env.advance(1500);
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PAUSED });
    assert.equal(sync.debug().paused, true);
});

test('buffering and cued karaoke states never change the pause state', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    for (const state of [YT_STATE.UNSTARTED, YT_STATE.BUFFERING, YT_STATE.CUED]) {
        sync.on_message(false, { event: 'onStateChange', info: state });
    }
    assert.equal(sync.debug().paused, false);
    assert.deepEqual(env.sent, []);
});

test('set_mapping: re-aims a running guide at once, keeps the vocal on', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(-10, 1);
    sync.set_start(60);
    report(sync, 60, 50);
    sync.switch_vocal(true);
    env.clear();

    sync.set_mapping(-9.9, 1);
    const [seek] = env.commands(FRAME.GUIDE, 'seekTo');
    assert.equal(seek.msg.args[0], 60 - 9.9 + LEAD.RESEEK.fallback);
    assert.equal(env.commands(FRAME.GUIDE, 'playVideo').length, 0, 'guide is already playing');
    const d = sync.debug();
    assert.equal(d.original, true);
    assert.equal(d.guide_offset, -9.9);

    // Same mapping again (e.g. Save after nudging): no re-seek
    env.clear();
    sync.set_mapping(-9.9, 1);
    assert.equal(env.posts.length, 0);
});

test('set_mapping: karaoke vocal only stores the mapping; intro target waits for tick', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(-10, 1);
    sync.set_start(60);
    env.clear();
    sync.set_mapping(-12, 1.001);
    assert.equal(env.posts.length, 0);
    assert.equal(sync.debug().guide_rate, 1.001);

    sync.set_start(2);
    sync.switch_vocal(true);
    env.clear();
    sync.set_mapping(-11, 1);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo').length, 0);
});

test('set_monitor: karaoke audio stays on under the guide until turned off or a new song', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.set_start(30);
    sync.switch_vocal(true);
    env.clear();

    sync.set_monitor(true);
    assert.equal(env.commands(FRAME.KARAOKE, 'unMute').length, 1);
    env.clear();
    sync.progress();
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 0, 'progress must not re-mute while monitoring');

    sync.switch_vocal(false);
    env.clear();
    sync.switch_vocal(true);
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 0, 'monitor survives a vocal toggle');

    sync.set_monitor(false);
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 1);

    sync.set_monitor(true);
    sync.load_song(0, 1);
    assert.equal(sync.debug().monitor, false);
});
