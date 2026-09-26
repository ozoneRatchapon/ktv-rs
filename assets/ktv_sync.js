// KTV-RS player sync core: drives the karaoke iframe and the original-vocal guide iframe (shown beside it while
// the original vocal is on) over the YouTube postMessage API and keeps the guide locked to karaoke time.
// Loaded by `src/sync.rs` (include_str + eval); unit-tested in Node by `tests/ktv_sync.test.cjs`.
// All browser access goes through the `env` passed to `create_sync`, so the core runs headless in tests.
(function (root) {
    'use strict';

    const FRAME = Object.freeze({ KARAOKE: 'ktv-youtube-player', GUIDE: 'ktv-guide-player' });
    const YT_STATE = Object.freeze({ UNSTARTED: -1, ENDED: 0, PLAYING: 1, PAUSED: 2, BUFFERING: 3, CUED: 5 });

    // Seek lead: YouTube resumes slightly late after seekTo, so the guide aims ahead. Latency differs by how the
    // guide restarts, so each kind learns its own lead from the first error after its seek; persisted across reloads.
    const LEAD = Object.freeze({
        RESEEK: { key: '_ktv_seek_lead', fallback: 0.3 }, // guide-only seek while both players run
        COLD: { key: '_ktv_cold_lead', fallback: 0.1 },   // guide starts from paused while karaoke keeps running
        PAIR: { key: '_ktv_pair_lead', fallback: 0.05 },  // karaoke and guide restart together (scrub, resume)
    });
    const LEAD_MAX = 1.5;

    const ACTION = Object.freeze({ NONE: 'none', HOLD: 'hold', RELEASE: 'release', RESEEK: 'reseek', RATE: 'rate' });
    const RESEEK_ERR_SECS = 1.0;  // beyond this, seek instead of nudging speed
    const SEEK_SETTLE_MS = 1500;  // ignore time reports right after a seek
    const CONTROL_MS = 250;
    // Karaoke state reports this soon after our own play/pause command may predate it; don't mirror them
    const OWN_COMMAND_MS = 1000;
    const PROGRESS_MS = 1000;

    // Speed nudge by error (YouTube only allows 0.05 steps). Between 0.03 and 0.08 the current rate is kept (hysteresis).
    function next_rate(err, rate_now) {
        if (err > 0.25) return 1.1;
        if (err < -0.25) return 0.9;
        if (err > 0.08) return 1.05;
        if (err < -0.08) return 0.95;
        if (Math.abs(err) < 0.03) return 1;
        return rate_now;
    }

    function clamp_lead(v) {
        return Math.min(LEAD_MAX, Math.max(0, v));
    }

    // err = target - guide after a seek aimed `lead` ahead; a large error means the seek itself failed, so skip learning
    function learn_lead(lead, err) {
        if (!(Math.abs(err) < RESEEK_ERR_SECS)) return lead;
        return clamp_lead(lead + err);
    }

    // A seek into a player that has never played (unknown, unstarted, cued, ended) pays a one-off buffering cost;
    // its landing error is not representative of the lead kind, so it must not train it
    function is_warm(state) {
        return state === YT_STATE.PLAYING || state === YT_STATE.PAUSED || state === YT_STATE.BUFFERING;
    }

    function parse_lead(raw) {
        if (raw === null || raw === undefined) return null;
        const v = parseFloat(raw);
        return v >= 0 && v <= LEAD_MAX ? v : null;
    }

    // Pure controller step. `target` is where the guide should be (MV seconds), `guide_now` where it is (null if unknown).
    function decide_guide(s) {
        if (!s.active || s.paused) return { kind: ACTION.NONE };
        // Karaoke intro is longer than the MV intro: hold the guide until the song starts
        if (s.target < 0) return { kind: ACTION.HOLD };
        if (s.waiting) return { kind: ACTION.RELEASE, target: s.target };
        if (s.settling || s.guide_now === null) return { kind: ACTION.NONE };
        const err = s.target - s.guide_now;
        if (Math.abs(err) > RESEEK_ERR_SECS) return { kind: ACTION.RESEEK, target: s.target, err };
        return { kind: ACTION.RATE, err, rate: next_rate(err, s.rate_now) };
    }

    // env: { now() ms monotonic, post(frame_id, message), has_frame(frame_id), send(msg), storage: { get(k), set(k, v) } }
    function create_sync(env) {
        const st = {
            send: env.send,
            guide_offset: 0,
            guide_rate: 1,
            original: false,
            paused: false,
            video_time: 0,
            video_at: null,
            start_sec: 0,
            mount_at: env.now(),
            guide_time: undefined,
            guide_at: null,
            guide_state: undefined,
            guide_rate_now: 1,
            guide_waiting: false,
            guide_hold_until: 0,
            guide_learn: null,
            guide_error: undefined,
            own_command_at: -Infinity,
            leads: {},
        };
        for (const kind of Object.keys(LEAD)) {
            const stored = parse_lead(env.storage.get(LEAD[kind].key));
            st.leads[kind] = stored === null ? LEAD[kind].fallback : stored;
        }

        function command(frame, func, args) {
            env.post(frame, JSON.stringify({ event: 'command', func, args: args || [] }));
        }

        // Karaoke time extrapolated from the last YouTube report (reports arrive every ~0.25-1s)
        function karaoke_time() {
            if (st.video_time > 0 && st.video_at !== null) {
                return st.paused ? st.video_time : st.video_time + (env.now() - st.video_at) / 1000;
            }
            return Math.max(0, (env.now() - st.mount_at) / 1000 + st.start_sec);
        }

        function guide_time() {
            if (typeof st.guide_time !== 'number' || st.guide_at === null) return null;
            if (st.paused || st.guide_state !== YT_STATE.PLAYING) return st.guide_time;
            return st.guide_time + (env.now() - st.guide_at) / 1000 * (st.guide_rate_now || 1);
        }

        // Measured mapping: MV time = offset + rate * karaoke time (negative during a longer karaoke intro)
        function guide_target(karaoke_sec) {
            return st.guide_offset + st.guide_rate * karaoke_sec;
        }

        function set_guide_rate(r) {
            if (st.guide_rate_now === r) return;
            st.guide_rate_now = r;
            command(FRAME.GUIDE, 'setPlaybackRate', [r]);
        }

        function seek_guide(target, lead_kind, play) {
            const dest = Math.max(0, target + st.leads[lead_kind]);
            st.guide_time = dest;
            st.guide_at = env.now();
            st.guide_hold_until = env.now() + SEEK_SETTLE_MS;
            st.guide_learn = is_warm(st.guide_state) ? lead_kind : null;
            set_guide_rate(1);
            command(FRAME.GUIDE, 'seekTo', [dest, true]);
            if (play) command(FRAME.GUIDE, 'playVideo');
        }

        function start_guide(lead_kind) {
            if (!st.original) return;
            seek_guide(guide_target(karaoke_time()), lead_kind, true);
        }

        function consume_learn(err) {
            const kind = st.guide_learn;
            if (!kind) return;
            st.guide_learn = null;
            const next = learn_lead(st.leads[kind], err);
            if (next === st.leads[kind]) return;
            st.leads[kind] = next;
            env.storage.set(LEAD[kind].key, String(next));
        }

        // Closed-loop sync step (every CONTROL_MS)
        function tick() {
            const action = decide_guide({
                active: st.original,
                paused: st.paused,
                target: guide_target(karaoke_time()),
                guide_now: guide_time(),
                waiting: st.guide_waiting,
                settling: env.now() < st.guide_hold_until,
                rate_now: st.guide_rate_now,
            });
            switch (action.kind) {
                case ACTION.HOLD:
                    if (st.guide_state === YT_STATE.PLAYING) command(FRAME.GUIDE, 'pauseVideo');
                    st.guide_waiting = true;
                    break;
                case ACTION.RELEASE:
                    st.guide_waiting = false;
                    seek_guide(action.target, 'COLD', true);
                    break;
                case ACTION.RESEEK:
                    st.guide_error = action.err;
                    consume_learn(action.err);
                    seek_guide(action.target, 'RESEEK', false);
                    break;
                case ACTION.RATE:
                    st.guide_error = action.err;
                    consume_learn(action.err);
                    set_guide_rate(action.rate);
                    break;
                default:
                    break;
            }
            return action;
        }

        // Progress step (every PROGRESS_MS): keep YouTube reporting, report karaoke time to Rust
        function progress() {
            if (st.paused) return;
            env.post(FRAME.KARAOKE, '{"event":"listening"}');
            env.post(FRAME.GUIDE, '{"event":"listening"}');
            // Re-assert mute in case the karaoke iframe was remounted
            if (st.original) command(FRAME.KARAOKE, 'mute');
            st.send('TIME:' + Math.floor(karaoke_time()));
        }

        function on_message(from_guide, data) {
            const info = data && data.info;
            // Guide player messages must not drive karaoke time or end-of-song
            if (from_guide) {
                // Any guide error (removed, private, embedding disabled) means no vocal: fall back to karaoke audio
                if (data && data.event === 'onError') {
                    if (st.original) switch_vocal(false);
                    st.send('GUIDE_ERROR:' + (Number(data.info) || 0));
                    return;
                }
                if (info && typeof info.currentTime === 'number') {
                    st.guide_time = info.currentTime;
                    st.guide_at = env.now();
                }
                if (info && typeof info.playerState === 'number') st.guide_state = info.playerState;
                if (info && typeof info.playbackRate === 'number') st.guide_rate_now = info.playbackRate;
                return;
            }
            if (info && typeof info.currentTime === 'number') {
                st.video_time = info.currentTime;
                st.video_at = env.now();
            }
            if (data && data.info === YT_STATE.ENDED) st.send('ended');
            const state = data && typeof data.info === 'number' ? data.info : info && info.playerState;
            if (typeof state === 'number') on_karaoke_state(state);
        }

        // The karaoke player's own controls stay usable (YouTube forbids covering the player), so a pause or
        // play made there is mirrored: the guide follows and karaoke time stops extrapolating
        function on_karaoke_state(state) {
            if (env.now() - st.own_command_at < OWN_COMMAND_MS) return;
            if (state === YT_STATE.PAUSED && !st.paused) apply_paused(true, false);
            else if (state === YT_STATE.PLAYING && st.paused) apply_paused(false, false);
        }

        // New song: apply its measured guide mapping and reset guide state (vocal starts as karaoke).
        // The new karaoke iframe autoplays, so a pause carried over from the previous song is cleared.
        function load_song(offset_secs, rate) {
            st.guide_offset = offset_secs;
            st.guide_rate = rate;
            st.original = false;
            st.guide_waiting = false;
            st.guide_time = undefined;
            st.guide_at = null;
            st.guide_state = undefined;
            st.guide_learn = null;
            st.guide_rate_now = 1;
            if (st.paused) {
                st.paused = false;
                st.send('PAUSE_STATE:0');
            }
        }

        // Karaoke iframe (re)mounted at `sec`
        function set_start(sec) {
            st.mount_at = env.now();
            st.start_sec = sec;
            st.video_time = sec;
            st.video_at = env.now();
        }

        function seek_all(target) {
            set_start(target);
            command(FRAME.KARAOKE, 'seekTo', [target, true]);
            if (st.original) seek_guide(guide_target(target), 'PAIR', false);
        }

        function seek_by(delta) {
            seek_all(Math.max(0, Math.floor(karaoke_time() + delta)));
        }

        function set_paused(paused) {
            if (!env.has_frame(FRAME.KARAOKE)) return;
            apply_paused(paused, true);
        }

        // `drive_karaoke` is false when the karaoke player itself reported the change
        function apply_paused(paused, drive_karaoke) {
            if (paused && !st.paused) {
                // Freeze extrapolated time at the pause moment
                st.video_time = karaoke_time();
                st.video_at = env.now();
            }
            st.paused = paused;
            if (!paused) {
                st.mount_at = env.now();
                st.start_sec = st.video_time || 0;
                st.video_at = env.now();
            }
            if (drive_karaoke) {
                st.own_command_at = env.now();
                command(FRAME.KARAOKE, paused ? 'pauseVideo' : 'playVideo');
            }
            if (paused) {
                command(FRAME.GUIDE, 'pauseVideo');
            } else {
                start_guide('PAIR');
            }
            st.send('PAUSE_STATE:' + (paused ? '1' : '0'));
        }

        // Replay: back to `sec` and playing, whether or not the song was paused
        function restart(sec) {
            if (st.paused) set_paused(false);
            seek_all(sec);
        }

        function toggle_playback() {
            set_paused(!st.paused);
        }

        function switch_vocal(original) {
            st.original = original;
            st.guide_waiting = false;
            if (original) {
                // Mute karaoke backing audio (video and lyrics stay visible), unmute and sync the guide
                command(FRAME.KARAOKE, 'mute');
                command(FRAME.GUIDE, 'unMute');
                command(FRAME.GUIDE, 'setVolume', [100]);
                if (st.paused) {
                    seek_guide(guide_target(karaoke_time()), 'PAIR', false);
                } else {
                    start_guide('COLD');
                }
            } else {
                command(FRAME.GUIDE, 'mute');
                command(FRAME.GUIDE, 'pauseVideo');
                command(FRAME.KARAOKE, 'unMute');
                command(FRAME.KARAOKE, 'setVolume', [100]);
            }
        }

        function bind(send) {
            st.send = send;
        }

        // Snapshot for devtools / tests: `KtvSync.debug()`
        function debug() {
            return {
                karaoke_time: karaoke_time(),
                guide_time: guide_time(),
                guide_error: st.guide_error,
                guide_state: st.guide_state,
                guide_rate_now: st.guide_rate_now,
                original: st.original,
                paused: st.paused,
                leads: Object.assign({}, st.leads),
            };
        }

        return {
            bind, load_song, set_start, seek_all, seek_by, restart, set_paused, toggle_playback, switch_vocal,
            tick, progress, on_message, debug,
        };
    }

    // Browser wiring; idempotent so Player remounts only rebind the Rust channel
    function install(win, send) {
        if (win.KtvSync) {
            win.KtvSync.bind(send);
            return win.KtvSync;
        }
        const frame = (id) => {
            const f = win.document.getElementById(id);
            return f && f.contentWindow ? f : null;
        };
        const sync = create_sync({
            now: () => win.performance.now(),
            send,
            has_frame: (id) => frame(id) !== null,
            post: (id, msg) => {
                const f = frame(id);
                if (!f) return;
                try { f.contentWindow.postMessage(msg, '*'); } catch { /* frame navigating */ }
            },
            storage: {
                get: (k) => { try { return win.localStorage.getItem(k); } catch { return null; } },
                set: (k, v) => { try { win.localStorage.setItem(k, v); } catch { /* storage disabled */ } },
            },
        });
        win.addEventListener('message', (event) => {
            let data;
            try {
                data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data;
            } catch {
                return; // not a YouTube message
            }
            const guide = frame(FRAME.GUIDE);
            sync.on_message(guide !== null && event.source === guide.contentWindow, data);
        });
        win.setInterval(sync.tick, CONTROL_MS);
        win.setInterval(sync.progress, PROGRESS_MS);
        win.KtvSync = sync;
        return sync;
    }

    const api = Object.freeze({
        FRAME, YT_STATE, LEAD, ACTION, next_rate, learn_lead, is_warm, parse_lead, decide_guide, create_sync, install,
    });
    if (typeof module === 'object' && module.exports) module.exports = api;
    root.KtvSyncCore = api;
})(typeof window !== 'undefined' ? window : globalThis);
