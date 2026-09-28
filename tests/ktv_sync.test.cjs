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
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: karaoke_sec, playerState: YT_STATE.PLAYING } });
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

test('time() is the karaoke clock the debug snapshot reports (mic frames read it ~47 times a second)', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    report(sync, 10, 0);
    env.advance(250);
    assert.equal(sync.time(), 10.25);
    assert.equal(sync.time(), sync.debug().karaoke_time);
    sync.set_paused(true);
    env.advance(1000);
    assert.equal(sync.time(), 10.25, 'paused: the clock stands still');
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

    // The karaoke player ends the song (after playing, as a real player always reports first)
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PLAYING });
    env.clear();
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.deepEqual(env.sent, ['ended']);
});

test('end of song: reported once, whether YouTube says it in infoDelivery, onStateChange or both', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.on_message(false, { event: 'infoDelivery', info: { playerState: YT_STATE.PLAYING, currentTime: 200 } });
    // Without an onStateChange subscription YouTube only puts the end in infoDelivery (seen on real embeds)
    sync.on_message(false, { event: 'infoDelivery', info: { playerState: YT_STATE.ENDED, currentTime: 217 } });
    assert.deepEqual(env.sent.filter((m) => m === 'ended'), ['ended'], 'infoDelivery alone ends the song');
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: 217 } });
    assert.deepEqual(env.sent.filter((m) => m === 'ended'), ['ended'], 'one ending, one next song');
    // Replay after the end: the next ending counts again
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PLAYING });
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.deepEqual(env.sent.filter((m) => m === 'ended'), ['ended', 'ended']);
});

test('the next song is not ended by the last one: a late end message from the old player is ignored', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.on_message(false, { event: 'infoDelivery', info: { playerState: YT_STATE.PLAYING, currentTime: 216 } });
    sync.on_message(false, { event: 'infoDelivery', info: { playerState: YT_STATE.ENDED } });
    // Rust moves on and loads the next song, then the old player's onStateChange for the same end arrives
    sync.load_song(0, 1);
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.deepEqual(env.sent.filter((m) => m === 'ended'), ['ended'], 'one ending must not skip two songs');
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.PLAYING });
    sync.on_message(false, { event: 'onStateChange', info: YT_STATE.ENDED });
    assert.deepEqual(env.sent.filter((m) => m === 'ended'), ['ended', 'ended'], 'the new song ends normally');
});

test('each karaoke player that loads is asked for its state changes (YouTube sends them only when asked)', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.on_message(false, { event: 'initialDelivery', info: { playerState: YT_STATE.UNSTARTED } });
    assert.deepEqual(env.commands(FRAME.KARAOKE, 'addEventListener').map((p) => p.msg.args), [['onStateChange']]);
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: 3 } });
    assert.equal(env.commands(FRAME.KARAOKE, 'addEventListener').length, 1, 'once per load, not per message');
    sync.on_message(true, { event: 'initialDelivery', info: {} });
    assert.equal(env.commands(FRAME.GUIDE, 'addEventListener').length, 0, 'the guide never ends a song');
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

// YouTube reports karaoke player state as onStateChange (info is the state) or inside infoDelivery
function karaoke_state(sync, state) {
    sync.on_message(false, { event: 'onStateChange', info: state });
}

test('karaoke clock waits for the player: blocked autoplay does not run the time on', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(18);
    // What a blocked player sends: position 0 (it has not moved to the start second yet), before and after its state
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: 0 } });
    karaoke_state(sync, YT_STATE.UNSTARTED);
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: 0, playerState: YT_STATE.UNSTARTED } });
    sync.on_message(false, { event: 'infoDelivery', info: { currentTime: 0 } });
    env.advance(2000);
    assert.equal(sync.debug().karaoke_time, 18, 'unstarted: time stays at the start second');
    sync.progress();
    assert.deepEqual(env.sent, ['TIME:18']);
});

test('blocked autoplay shows as paused after the wait; Play starts the video and the clock', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(18);
    karaoke_state(sync, YT_STATE.UNSTARTED);
    env.advance(2000);
    sync.tick();
    assert.equal(sync.debug().paused, false, 'still inside the autoplay wait');
    env.advance(1000);
    sync.tick();
    assert.equal(sync.debug().paused, true);
    assert.ok(env.sent.includes('PAUSE_STATE:1'), 'Rust shows Play');
    assert.equal(env.commands(FRAME.KARAOKE, 'pauseVideo').length, 0, 'the karaoke player is not commanded');

    env.clear();
    sync.toggle_playback();
    assert.equal(env.commands(FRAME.KARAOKE, 'playVideo').length, 1);
    assert.deepEqual(env.sent, ['PAUSE_STATE:0']);
    env.advance(500);
    karaoke_state(sync, YT_STATE.PLAYING);
    env.advance(3000);
    assert.equal(Math.round(sync.debug().karaoke_time), 21, 'runs from when it started playing');
});

test('YouTube play button on a blocked video is mirrored: unpaused, clock runs', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(0);
    karaoke_state(sync, YT_STATE.CUED);
    env.advance(3000);
    sync.tick();
    assert.equal(sync.debug().paused, true);
    karaoke_state(sync, YT_STATE.PLAYING);
    assert.equal(sync.debug().paused, false);
    env.advance(4000);
    assert.equal(Math.round(sync.debug().karaoke_time), 4);
});

test('autoplay that works never pauses; buffering holds the clock; a new mount forgets the old state', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(10);
    karaoke_state(sync, YT_STATE.UNSTARTED);
    env.advance(300);
    karaoke_state(sync, YT_STATE.BUFFERING);
    env.advance(3000);
    sync.tick();
    assert.equal(sync.debug().paused, false, 'buffering is loading, not blocked');
    assert.equal(sync.debug().karaoke_time, 10, 'buffering: clock held');
    karaoke_state(sync, YT_STATE.PLAYING);
    env.advance(2000);
    assert.equal(Math.round(sync.debug().karaoke_time), 12);

    sync.set_start(0);
    assert.equal(sync.debug().karaoke_state, undefined);
});

test('practice loop: past the end it jumps back to the start; a new song or a clear ends it', () => {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.set_start(0);
    report(sync, 60);
    sync.set_loop(60, 75);
    env.advance(10000);
    sync.tick();
    assert.equal(env.commands(FRAME.KARAOKE, 'seekTo').length, 0, 'inside the loop: no seek');
    env.advance(6000);
    sync.tick();
    assert.deepEqual(env.commands(FRAME.KARAOKE, 'seekTo').map((p) => p.msg.args), [[60, true]]);
    assert.equal(Math.round(sync.debug().karaoke_time), 60);

    sync.set_loop(-1, -1);
    assert.equal(sync.debug().loop, null);
    sync.set_loop(80, 70);
    assert.equal(sync.debug().loop, null, 'end before start is no loop');
    sync.set_loop(10, 20);
    sync.load_song(0, 1);
    assert.equal(sync.debug().loop, null, 'a new song clears the loop');
});

// Vocal on and warm, then the guide stops following seeks (an ad plays in its place): each tick past the settle
// window finds it several seconds behind
function lost_guide_setup() {
    const env = fake_env();
    const sync = core.create_sync(env);
    sync.load_song(0, 1);
    sync.switch_vocal(true);
    env.advance(2000);
    report(sync, 20, 20);
    sync.tick();
    return { env, sync };
}

function miss(env, sync, karaoke_sec) {
    env.advance(1600); // past SEEK_SETTLE_MS
    report(sync, karaoke_sec, 3); // guide reports ad time, not the MV position
    return sync.tick();
}

test('guide lost: after LOST_RESEEKS missed re-seeks the guide is muted and karaoke audio plays', () => {
    const { env, sync } = lost_guide_setup();
    env.clear();
    for (let i = 1; i < core.LOST_RESEEKS; i += 1) {
        assert.equal(miss(env, sync, 20 + i * 2).kind, ACTION.RESEEK);
        assert.equal(sync.debug().guide_lost, false, `not lost after ${i} misses`);
    }
    assert.equal(env.commands(FRAME.KARAOKE, 'unMute').length, 0);

    miss(env, sync, 30);
    assert.equal(sync.debug().guide_lost, true);
    assert.equal(sync.debug().original, true, 'the vocal choice is kept');
    assert.equal(env.commands(FRAME.GUIDE, 'mute').length, 1);
    assert.equal(env.commands(FRAME.KARAOKE, 'unMute').length, 1);
    assert.deepEqual(env.sent, ['GUIDE_LOST:1']);

    // Still lost: no repeated commands or messages, and progress no longer re-mutes karaoke
    env.clear();
    miss(env, sync, 32);
    sync.progress();
    assert.equal(env.commands(FRAME.GUIDE, 'mute').length, 0);
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 0);
    assert.deepEqual(env.sent.filter((m) => m.startsWith('GUIDE_LOST')), []);
    assert.equal(env.commands(FRAME.GUIDE, 'seekTo').length, 1, 'keeps trying to re-seek');
});

test('guide lost: back in step restores the original vocal and tells Rust', () => {
    const { env, sync } = lost_guide_setup();
    for (let i = 0; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 22 + i * 2);
    env.clear();

    env.advance(1600);
    report(sync, 40, 40);
    assert.equal(sync.tick().kind, ACTION.RATE);
    assert.equal(sync.debug().guide_lost, false);
    assert.equal(env.commands(FRAME.GUIDE, 'unMute').length, 1);
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 1);
    assert.deepEqual(env.sent, ['GUIDE_LOST:0']);
});

test('guide lost: an in-step tick or a user seek restarts the count', () => {
    const { env, sync } = lost_guide_setup();
    for (let i = 1; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 20 + i * 2);
    env.advance(1600);
    report(sync, 30, 30);
    sync.tick();
    for (let i = 1; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 30 + i * 2);
    assert.equal(sync.debug().guide_lost, false, 'misses around an in-step tick do not add up');

    sync.seek_all(90);
    miss(env, sync, 92);
    assert.equal(sync.debug().guide_lost, false, 'a scrub starts a fresh count');
});

test('guide lost: timing mode keeps karaoke audio on when the guide comes back', () => {
    const { env, sync } = lost_guide_setup();
    sync.set_monitor(true);
    for (let i = 0; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 22 + i * 2);
    sync.set_monitor(false);
    env.clear();
    sync.progress();
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 0, 'monitor off while lost keeps karaoke audible');

    sync.set_monitor(true);
    env.advance(1600);
    report(sync, 40, 40);
    sync.tick();
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 0);
    assert.equal(env.commands(FRAME.GUIDE, 'unMute').length, 1);
});

test('guide lost: switching the vocal or a new song clears it', () => {
    const { env, sync } = lost_guide_setup();
    for (let i = 0; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 22 + i * 2);
    sync.switch_vocal(false);
    assert.equal(sync.debug().guide_lost, false);
    sync.switch_vocal(true);
    env.clear();
    sync.progress();
    assert.equal(env.commands(FRAME.KARAOKE, 'mute').length, 1, 'vocal back on mutes karaoke again');

    for (let i = 0; i < core.LOST_RESEEKS; i += 1) miss(env, sync, 50 + i * 2);
    assert.equal(sync.debug().guide_lost, true);
    sync.load_song(0, 1);
    assert.equal(sync.debug().guide_lost, false);
});
