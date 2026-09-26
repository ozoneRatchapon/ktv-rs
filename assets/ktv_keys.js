// KTV booth keyboard: type-to-search, Space pause, arrows seek, ? help.
// Classic script in the static <head> (asset! in src/main.rs); Rust calls `KtvKeysCore.install` via src/js_bridge.rs.
// Unit-tested in Node by `tests/ktv_keys.test.cjs`.
// Messages to Rust are parsed by `KeyAction::parse` in `src/keys.rs`: keep the two in step.
(function (root) {
    'use strict';

    const SEEK_STEP_SECS = 5;

    /** Where keyboard focus is, as far as booth keys care. */
    const FOCUS = { TEXT: 'text', CONTROL: 'control', OTHER: 'other' };

    /** Message for a keydown, or null to leave the key alone (`focus`: a FOCUS value). */
    function key_action(e, focus) {
        if (focus === FOCUS.TEXT || e.ctrlKey || e.metaKey || e.altKey) return null;
        // A control reached with the keyboard keeps Space/Enter (activate it); after a mouse click
        // Space stays the booth's pause key.
        if (focus === FOCUS.CONTROL && (e.key === ' ' || e.key === 'Enter')) return null;
        switch (e.key) {
            case 'Escape': return { msg: 'ESC', prevent: false };
            case '?': return { msg: 'HELP', prevent: false };
            case ' ': return { msg: 'SPACE', prevent: true };
            case 'Backspace':
            case 'Delete': return { msg: 'BACKSPACE', prevent: true };
            case 'ArrowLeft': return { msg: 'SEEK_REL:' + -SEEK_STEP_SECS, prevent: true };
            case 'ArrowRight': return { msg: 'SEEK_REL:' + SEEK_STEP_SECS, prevent: true };
            default: return e.key.length === 1 ? { msg: 'CHAR:' + e.key, prevent: false } : null;
        }
    }

    /** `by_keyboard`: focus arrived without a pointer press (Tab, or script). Not `:focus-visible`: Chrome
     *  turns that on for a mouse-clicked button as soon as any key is pressed on it. */
    function focus_kind(el, by_keyboard) {
        if (!el) return FOCUS.OTHER;
        const tag = el.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable === true) return FOCUS.TEXT;
        const control = tag === 'BUTTON' || tag === 'A' || tag === 'SUMMARY' || (el.getAttribute && el.getAttribute('role') === 'button');
        return control && by_keyboard ? FOCUS.CONTROL : FOCUS.OTHER;
    }

    /** Wire the listeners once per page; a remount only rebinds the Rust channel. */
    function install(win, send) {
        if (win.KtvKeys) {
            win.KtvKeys.bind(send);
            return win.KtvKeys;
        }
        let emit = send;
        const doc = win.document;
        // Buttons activate on Space's keyup: when the booth took the keydown, cancel the keyup too,
        // or a mouse-clicked button (e.g. Next Song) would also fire.
        let took_space = false;
        // How the focused element got focus: a pointer press just before, or not
        let pointer_down = false;
        let focus_by_keyboard = true;
        win.addEventListener('pointerdown', () => { pointer_down = true; }, true);
        win.addEventListener('focusin', () => {
            focus_by_keyboard = !pointer_down;
            pointer_down = false;
        }, true);
        win.addEventListener('keydown', (e) => {
            const action = key_action(e, focus_kind(doc.activeElement, focus_by_keyboard));
            if (e.key === ' ') took_space = action !== null;
            if (!action) return;
            if (action.prevent) e.preventDefault();
            emit(action.msg);
        });
        win.addEventListener('keyup', (e) => {
            if (e.key === ' ' && took_space) {
                e.preventDefault();
                took_space = false;
            }
        });
        // A click inside a YouTube iframe moves keyboard focus into it, which would swallow
        // type-to-search. The click has already landed by the time blur fires, so take focus back.
        win.addEventListener('blur', () => win.setTimeout(() => {
            const el = doc.activeElement;
            if (el && el.tagName === 'IFRAME') {
                el.blur();
                win.focus();
            }
        }, 0));
        win.KtvKeys = { bind: (next) => { emit = next; } };
        return win.KtvKeys;
    }

    const api = { SEEK_STEP_SECS, FOCUS, key_action, focus_kind, install };
    if (typeof module === 'object' && module.exports) module.exports = api;
    root.KtvKeysCore = api;
})(typeof window !== 'undefined' ? window : globalThis);
