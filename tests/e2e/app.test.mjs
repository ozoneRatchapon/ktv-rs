// End-to-end checks of the release build in headless Chrome.
// Run: tools/build_web.sh && npx wrangler dev --port 8788, then `node --test tests/e2e/app.test.mjs`.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { launch, sleep } from './cdp.mjs';

let browser;
before(async () => { browser = await launch(); });
after(async () => { await browser?.close(); });

/** Fresh page (own storage) on the app; checks no errors / CSP violations when the test is done. */
async function with_page(opts, body) {
  const page = await browser.new_page(opts);
  try {
    await page.goto();
    await body(page);
    assert.deepEqual(await page.eval('window.__csp'), [], 'CSP violations');
    assert.deepEqual(page.errors, [], 'console errors');
  } finally {
    await page.close();
  }
}

const help_open = `!!document.querySelector('.shortcut-help')`;
const search_value = `document.getElementById('song_search').value`;

test('page shell: lang, title, favicon, named form fields', () => with_page({}, async (page) => {
  assert.equal(await page.eval('document.documentElement.lang'), 'en');
  assert.equal(await page.eval('document.title'), 'KTV-RS · Thai Karaoke');
  assert.ok(await page.eval(`document.querySelectorAll('.song-title[lang=th]').length > 0`), 'song titles tagged Thai');
  assert.equal(await page.eval(`fetch(document.querySelector('link[rel=icon]').href).then((r) => r.status)`), 200);
  assert.equal(await page.eval(`[...document.querySelectorAll('input, select, textarea')].filter((e) => !e.id || !e.name).length`), 0);
}));

test('type-to-search works, including after focus moved into a player iframe', () => with_page({}, async (page) => {
  await page.key('k');
  await page.wait_for(`${search_value} === 'k'`);
  const focus = await page.eval(`(async () => {
    const f = document.createElement('iframe');
    document.body.appendChild(f);
    await new Promise((r) => setTimeout(r, 100));
    f.contentWindow.focus();
    f.focus();
    const during = document.activeElement.tagName;
    await new Promise((r) => setTimeout(r, 100));
    const after = document.activeElement.tagName;
    f.remove();
    return during + '>' + after;
  })()`);
  assert.equal(focus, 'IFRAME>BODY', 'focus handed back to the page');
  await page.key('t');
  await page.wait_for(`${search_value} === 'kt'`);
  await page.key('Backspace');
  await page.wait_for(`${search_value} === 'k'`);
  await page.key('Escape');
  await page.wait_for(`${search_value} === ''`);
}));

test('shortcut help: open on first visit, ? toggles without typing, Close is remembered', () => with_page({}, async (page) => {
  assert.ok(await page.eval(help_open), 'open on a first visit');
  await page.key('?');
  await page.wait_for(`!${help_open}`);
  await page.key('?');
  await page.wait_for(help_open);
  assert.equal(await page.eval(search_value), '', '? is not typed into search');
  await page.eval(`[...document.querySelectorAll('.shortcut-help button')].find((b) => b.textContent === 'Close').click()`);
  await page.wait_for(`!${help_open}`);
  await page.wait_for(`JSON.parse(localStorage.getItem('ktv.settings.v1')).seen_shortcuts === true`);
  await page.reload();
  assert.equal(await page.eval(help_open), false, 'stays closed after reload');
  await page.click('button[aria-label="Keyboard shortcuts"]');
  await page.wait_for(help_open);
}));

/** Seed a device-saved guide timing for the current song (optionally turning it into an Add URL song). */
const seed_saved_timing = (custom) => `(() => {
  const settings = JSON.parse(localStorage.getItem('ktv.settings.v1'));
  settings.show_timing_tools = true;
  localStorage.setItem('ktv.settings.v1', JSON.stringify(settings));
  const s = JSON.parse(localStorage.getItem('ktv.session.v1'));
  const guide = Object.assign({}, s.current.song.guide, { offset_secs: 1.5 });
  if (${custom}) {
    const song = Object.assign({}, s.current.song, { id: 'custom_e2e', code: '90001', title: 'E2E Song', youtube_id: 'AAAAAAAAAAA' });
    s.current.song = song;
    s.custom_songs = [song];
  }
  localStorage.setItem('ktv.session.v1', JSON.stringify(s));
  localStorage.setItem('ktv.guides.v1', JSON.stringify({ [s.current.song.id]: guide }));
})()`;

for (const [kind, custom, notice, status] of [
  ['built-in song', false, 'Back to the catalog timing', 'from catalog'],
  ['Add URL song', true, 'Guide removed: this song has no catalog guide', 'no guide yet'],
]) {
  test(`Revert says what it does: ${kind}`, () => with_page({}, async (page) => {
    await page.wait_for(`localStorage.getItem('ktv.session.v1') && localStorage.getItem('ktv.settings.v1')`);
    await page.eval(seed_saved_timing(custom));
    await page.reload();
    await page.wait_for(`document.querySelector('.timing-status')?.textContent === 'saved on this device'`);
    const share = await page.eval(`[...document.querySelectorAll('.guide-timing-panel a')].find((a) => a.textContent === 'Share on GitHub')?.href`);
    assert.match(share, /^https:\/\/github\.com\/ozoneRatchapon\/ktv-rs\/issues\/new\?template=guide_timing\.yml&/);
    await page.eval(`[...document.querySelectorAll('.guide-timing-panel button')].find((b) => b.textContent === 'Revert').click()`);
    await page.wait_for(`document.querySelector('.timing-notice')?.textContent === ${JSON.stringify(notice)}`);
    assert.equal(await page.eval(`document.querySelector('.timing-status').textContent`), status);
    await page.wait_for(`localStorage.getItem('ktv.guides.v1') === '{}'`);
  }));
}

for (const width of [1280, 900, 600, 375]) {
  test(`every player control is reachable at ${width}px`, () => with_page({ width }, async (page) => {
    await sleep(300);
    const clipped = await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')]
      .filter((b) => { const r = b.getBoundingClientRect(); return r.width === 0 || r.left < 0 || r.right > innerWidth; })
      .map((b) => b.textContent)`);
    assert.deepEqual(clipped, []);
    assert.ok(await page.eval(`document.querySelector('.player-main-controls-row button') !== null`), 'controls rendered');
    assert.ok(await page.eval(`document.querySelector('.song-titles').getBoundingClientRect().width >= 100`), 'title not squeezed');
    assert.ok(await page.eval(`document.documentElement.scrollWidth <= innerWidth`), 'no sideways page scroll');
  }));
}

/** Click a song card's button (Play / Insert / Queue) by keypad code; returns the song title. */
const card_action = (code, button) => `(() => {
  const card = [...document.querySelectorAll('.song-card')].find((c) => c.querySelector('.song-code-tag')?.textContent === '#${code}');
  card.querySelector('.card-btn.${button}').click();
  return card.querySelector('.song-title').textContent;
})()`;
const now_title = `document.querySelector('.now-title')?.textContent`;
const queue_titles = `(async () => {
  [...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click();
  await new Promise((r) => setTimeout(r, 200));
  const titles = [...document.querySelectorAll('.item-title')].map((e) => e.textContent);
  [...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Songbook').click();
  await new Promise((r) => setTimeout(r, 200));
  return titles;
})()`;

test('queue: Queue appends, Insert goes first, Play replaces, Next Song advances, all survive reload', () => with_page({}, async (page) => {
  const start = await page.eval(queue_titles);
  assert.equal(start.length, 3, 'demo queue');
  const queued = await page.eval(card_action('10001', 'queue-add'));
  const inserted = await page.eval(card_action('10005', 'queue-next'));
  await sleep(200);
  assert.deepEqual(await page.eval(queue_titles), [inserted, ...start, queued]);
  const played = await page.eval(card_action('10008', 'play-now'));
  await page.wait_for(`${now_title} === ${JSON.stringify(played)}`);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`${now_title} === ${JSON.stringify(inserted)}`);
  assert.deepEqual(await page.eval(queue_titles), [...start, queued]);
  await page.reload();
  assert.equal(await page.eval(now_title), inserted);
  assert.deepEqual(await page.eval(queue_titles), [...start, queued]);
}));

test('empty queue: Next Song hands over to Auto-DJ with a different song', () => with_page({}, async (page) => {
  const finished = await page.eval(now_title);
  await page.eval(`(async () => {
    [...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click();
    await new Promise((r) => setTimeout(r, 200));
    [...document.querySelectorAll('button')].find((b) => b.textContent === 'Clear Queue').click();
  })()`);
  await page.wait_for(`document.querySelectorAll('.item-title').length === 0`);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`document.querySelector('.auto-dj-toast')?.textContent.includes('Auto-DJ')`);
  const next = await page.eval(now_title);
  assert.ok(next && next !== finished, `Auto-DJ picked ${next} after ${finished}`);
}));

test('mic: room check, noise gate, held notes give a Tuning score, the finished song shows a result card and joins Recent scores', () => with_page({ fake_mic: true }, async (page) => {
  const sung = await page.eval(now_title);
  await page.eval(`[...document.querySelectorAll('.pitch-meter button')].find((b) => b.textContent.startsWith('Mic')).click()`);
  await page.wait_for(`!!window.__mic_gain`);
  // Noisy room during the check: sawtooth RMS = gain / sqrt(3), so 0.03 -> ~0.017 RMS -> gate ~0.035
  await page.eval(`window.__mic_gain.gain.value = 0.03`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent === 'Room check…'`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent !== 'Room check…'`, 5000);
  // ~0.023 RMS: above the detector's fixed 0.01 floor, below twice the room -> ignored
  await page.eval(`window.__mic_gain.gain.value = 0.04`);
  await sleep(400);
  assert.equal(await page.eval(`document.querySelector('.pitch-note').textContent`), '—', 'room-level sound is gated');
  await page.eval(`window.__mic_gain.gain.value = 0.3`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent === 'A3'`);
  // Four in-tune held notes (A3, B3, C4, D4), ~0.5 s each
  for (const hz of [220, 246.94, 261.63, 293.66]) {
    await page.eval(`window.__mic.frequency.value = ${hz}`);
    await sleep(500);
  }
  await page.wait_for(`/^\\d+$/.test(document.querySelector('.tuning-value')?.textContent ?? '')`);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`!!document.querySelector('.score-card')`);
  const card = await page.eval(`JSON.stringify({
    value: document.querySelector('.score-card-value').textContent,
    song: document.querySelector('.score-card-song').textContent,
  })`);
  const { value, song } = JSON.parse(card);
  assert.ok(Number(value) >= 90, `sawtooth in tune scores high, got ${value}`);
  assert.ok(song.startsWith(sung), `card names the finished song: ${song}`);
  await page.reload();
  const rows = await page.eval(`(async () => {
    [...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click();
    await new Promise((r) => setTimeout(r, 200));
    return [...document.querySelectorAll('.score-row-song')].map((e) => e.textContent);
  })()`);
  assert.deepEqual(rows, [sung], 'history survives reload');
}));

test('search: romanised alias and wrong keyboard layout both find Thai songs', () => with_page({}, async (page) => {
  const titles = `[...document.querySelectorAll('.song-title')].map((e) => e.textContent)`;
  for (const key of 'rak mai wai'.replace(/ /g, '')) await page.key(key);
  await page.wait_for(`${titles}.length === 1`);
  assert.deepEqual(await page.eval(titles), ['รักไม่ไหวแล้วโว้ย']);
  await page.key('Escape');
  // "รัก" typed with the keyboard left on English
  for (const key of 'iyd') await page.key(key);
  await page.wait_for(`!!document.querySelector('.retyped-text')`);
  assert.equal(await page.eval(`document.querySelector('.retyped-text strong').textContent`), 'รัก');
  assert.ok(await page.eval(`${titles}.length > 0 && ${titles}.every((t) => t.includes('รัก'))`));
}));

test('favourites and recently sung shelves', () => with_page({}, async (page) => {
  const chip = (label) => `[...document.querySelectorAll('.chip')].find((c) => c.textContent === ${JSON.stringify(label)})`;
  const titles = `[...document.querySelectorAll('.song-title')].map((e) => e.textContent)`;
  const starred = await page.eval(`(() => {
    const card = [...document.querySelectorAll('.song-card')].find((c) => c.querySelector('.song-code-tag').textContent === '#10005');
    card.querySelector('.fav-btn').click();
    return card.querySelector('.song-title').textContent;
  })()`);
  await page.eval(`${chip('★ Favourites')}.click()`);
  await page.wait_for(`JSON.stringify(${titles}) === ${JSON.stringify(JSON.stringify([starred]))}`);
  // Recently sung: newest first (seeded; a real entry needs 30 s on stage)
  await page.eval(`localStorage.setItem('ktv.picks.v1', JSON.stringify({ favourites: ['gmm_005'], recent: ['gmm_003', 'gmm_001'] }))`);
  await page.reload();
  await page.eval(`${chip('Recently sung')}.click()`);
  await page.wait_for(`${titles}.length === 2`);
  assert.deepEqual(await page.eval(`[...document.querySelectorAll('.song-code-tag')].map((e) => e.textContent)`), ['#10003', '#10001']);
  await page.eval(`${chip('★ Favourites')}.click()`);
  await page.wait_for(`JSON.stringify(${titles}) === ${JSON.stringify(JSON.stringify([starred]))}`);
}));
