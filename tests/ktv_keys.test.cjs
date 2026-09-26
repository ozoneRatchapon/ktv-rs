// Unit tests for the booth keyboard (assets/ktv_keys.js). Run: node --test tests/ktv_keys.test.cjs
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const keys = require('../assets/ktv_keys.js');

const press = (key, mods = {}, typing = false) => keys.key_action({ key, ...mods }, typing);

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
    assert.equal(press('a', {}, true), null);
    assert.equal(press(' ', {}, true), null, 'space in a text field types a space');
    assert.equal(press('c', { metaKey: true }), null, 'copy');
    assert.equal(press('r', { ctrlKey: true }), null, 'reload');
    assert.equal(press('a', { altKey: true }), null);
    for (const key of ['Shift', 'Tab', 'Enter', 'F5', 'ArrowUp']) assert.equal(press(key), null, key);
});

test('is_typing covers text fields and contenteditable only', () => {
    assert.equal(keys.is_typing(null), false);
    for (const tagName of ['INPUT', 'TEXTAREA', 'SELECT']) assert.equal(keys.is_typing({ tagName }), true, tagName);
    assert.equal(keys.is_typing({ tagName: 'DIV', isContentEditable: true }), true);
    assert.equal(keys.is_typing({ tagName: 'BUTTON' }), false);
    assert.equal(keys.is_typing({ tagName: 'IFRAME' }), false);
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
