// KTV booth keyboard: type-to-search, Space pause, arrows seek, ? help.
// Loaded by `src/keys.rs` (include_str + eval); unit-tested in Node by `tests/ktv_keys.test.cjs`.
// Messages to Rust are parsed by `KeyAction::parse` in `src/keys.rs`: keep the two in step.
(function (root) {
    'use strict';

    const SEEK_STEP_SECS = 5;

    /** Message for a keydown, or null to leave the key alone. `typing` = focus is in a text field. */
    function key_action(e, typing) {
        if (typing || e.ctrlKey || e.metaKey || e.altKey) return null;
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

    function is_typing(el) {
        if (!el) return false;
        const tag = el.tagName;
        return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable === true;
    }

    /** Wire the listeners once per page; a remount only rebinds the Rust channel. */
    function install(win, send) {
        if (win.KtvKeys) {
            win.KtvKeys.bind(send);
            return win.KtvKeys;
        }
        let emit = send;
        const doc = win.document;
        win.addEventListener('keydown', (e) => {
            const action = key_action(e, is_typing(doc.activeElement));
            if (!action) return;
            if (action.prevent) e.preventDefault();
            emit(action.msg);
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

    const api = { SEEK_STEP_SECS, key_action, is_typing, install };
    if (typeof module === 'object' && module.exports) module.exports = api;
    root.KtvKeysCore = api;
})(typeof window !== 'undefined' ? window : globalThis);
