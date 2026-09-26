// Unit tests for the booth keyboard (assets/ktv_keys.js). Run: node --test tests/ktv_keys.test.cjs
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const keys = require('../assets/ktv_keys.js');

const { FOCUS } = keys;
const press = (key, mods = {}, focus = FOCUS.OTHER) => keys.key_action({ key, ...mods }, focus);

test('booth keys map to messages', () => {
    assert.deepEqual(press('Escape'), { msg: 'ESC', prevent: false });
    assert.deepEqual(press('?'), { msg: 'HELP', prevent: false });
    assert.deepEqual(press(' '), { msg: 'SPACE', prevent: true });
    assert.deepEqual(press('Backspace'), { msg: 'BACKSPACE', prevent: true });
    assert.deepEqual(press('Delete'), { msg: 'BACKSPACE', prevent: true });
    assert.deepEqual(press('ArrowLeft'), { msg: 'SEEK_REL:-5', prevent: true });
    assert.deepEqual(press('ArrowRight'), { msg: 'SEEK_REL:5', prevent: true });
    assert.deepEqual(press('a'), { msg: 'CHAR:a', prevent: false });
    assert.deepEqual(press('ก'), { msg: 'CHAR:ก', prevent: false });
});

test('keys are left alone while typing, with modifiers, or when not printable', () => {
    assert.equal(press('a', {}, FOCUS.TEXT), null);
    assert.equal(press(' ', {}, FOCUS.TEXT), null, 'space in a text field types a space');
    assert.equal(press('c', { metaKey: true }), null, 'copy');
    assert.equal(press('r', { ctrlKey: true }), null, 'reload');
    assert.equal(press('a', { altKey: true }), null);
    for (const key of ['Shift', 'Tab', 'Enter', 'F5', 'ArrowUp']) assert.equal(press(key), null, key);
});

test('focus_kind: text fields, keyboard-focused controls, everything else', () => {
    const el = (tagName, extra = {}) => ({ tagName, getAttribute: () => null, ...extra });
    assert.equal(keys.focus_kind(null, true), FOCUS.OTHER);
    for (const tag of ['INPUT', 'TEXTAREA', 'SELECT']) assert.equal(keys.focus_kind(el(tag), false), FOCUS.TEXT, tag);
    assert.equal(keys.focus_kind(el('DIV', { isContentEditable: true }), false), FOCUS.TEXT);
    for (const tag of ['BUTTON', 'A', 'SUMMARY']) assert.equal(keys.focus_kind(el(tag), true), FOCUS.CONTROL, tag);
    assert.equal(keys.focus_kind(el('BUTTON'), false), FOCUS.OTHER, 'mouse-clicked button');
    assert.equal(keys.focus_kind(el('DIV', { getAttribute: () => 'button' }), true), FOCUS.CONTROL, 'role=button');
    assert.equal(keys.focus_kind(el('IFRAME'), true), FOCUS.OTHER);
});

test('keyboard-focused controls keep Space and Enter, but still type into search', () => {
    assert.equal(press(' ', {}, FOCUS.CONTROL), null);
    assert.equal(press('Enter', {}, FOCUS.CONTROL), null);
    assert.deepEqual(press('a', {}, FOCUS.CONTROL), { msg: 'CHAR:a', prevent: false });
    assert.deepEqual(press(' ', {}, FOCUS.OTHER), { msg: 'SPACE', prevent: true }, 'after a mouse click Space still pauses');
});

// Fake window: records listeners, runs timers at once
function fake_win() {
    const listeners = {};
    const win = {
        document: { activeElement: null },
        focused: 0,
        addEventListener: (type, fn) => { (listeners[type] ??= []).push(fn); },
        setTimeout: (fn) => fn(),
        focus: () => { win.focused += 1; },
        fire: (type, e = {}) => (listeners[type] ?? []).forEach((fn) => fn(e)),
        count: (type) => (listeners[type] ?? []).length,
    };
    return win;
}

test('install wires listeners once, rebinds the channel on remount', () => {
    const win = fake_win();
    const first = [];
    const second = [];
    keys.install(win, (m) => first.push(m));
    keys.install(win, (m) => second.push(m));
    assert.equal(win.count('keydown'), 1);
    assert.equal(win.count('keyup'), 1);
    assert.equal(win.count('blur'), 1);
    let prevented = false;
    win.fire('keydown', { key: ' ', preventDefault: () => { prevented = true; } });
    assert.deepEqual([first, second], [[], ['SPACE']]);
    assert.equal(prevented, true);
});

test('focus comes back from an iframe, not from anything else', () => {
    const win = fake_win();
    keys.install(win, () => {});
    let blurred = 0;
    win.document.activeElement = { tagName: 'IFRAME', blur: () => { blurred += 1; } };
    win.fire('blur');
    assert.deepEqual([blurred, win.focused], [1, 1]);
    win.document.activeElement = { tagName: 'BODY', blur: () => { blurred += 1; } };
    win.fire('blur');
    assert.deepEqual([blurred, win.focused], [1, 1], 'switching apps leaves focus alone');
});

test('a Space the booth handled has its keyup cancelled (no click on a mouse-focused button)', () => {
    const win = fake_win();
    keys.install(win, () => {});
    const up = () => { let prevented = false; win.fire('keyup', { key: ' ', preventDefault: () => { prevented = true; } }); return prevented; };
    win.fire('keydown', { key: ' ', preventDefault: () => {} });
    assert.equal(up(), true);
    // Mouse-clicked button: Space is the booth's
    win.document.activeElement = { tagName: 'BUTTON', getAttribute: () => null };
    win.fire('pointerdown');
    win.fire('focusin');
    win.fire('keydown', { key: ' ', preventDefault: () => {} });
    assert.equal(up(), true);
    // Button reached with Tab: Space activates it, keyup left alone
    win.fire('focusin');
    win.fire('keydown', { key: ' ', preventDefault: () => {} });
    assert.equal(up(), false);
    // Typing a space in a field: keydown ignored, keyup left alone
    win.document.activeElement = { tagName: 'INPUT' };
    win.fire('keydown', { key: ' ', preventDefault: () => {} });
    assert.equal(up(), false);
});
