// End-to-end checks of the release build in headless Chrome.
// Run: tools/build_web.sh && npx wrangler dev --port 8788, then `node --test tests/e2e/app.test.mjs`.
import { after, before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
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
  assert.equal(await page.eval(`getComputedStyle(document.querySelector('.boot-splash')).display`), 'none', 'splash hidden once mounted');
  assert.ok(await page.eval(`fetch('/').then((r) => r.text()).then((html) => /<link rel="stylesheet" href="[^"]*main-[^"]*\.css"/.test(html))`), 'stylesheet in the static head');
  assert.equal(await page.eval(`fetch('/robots.txt').then((r) => r.text())`), 'User-agent: *\nAllow: /\n');
  // Link preview: the card is served as a PNG from this site's path (absolute URL points at prod)
  const og = await page.eval(`new URL(document.querySelector('meta[property="og:image"]').content).pathname`);
  assert.equal(og, '/og-image.png');
  assert.equal(await page.eval(`fetch('/og-image.png').then((r) => r.status + ' ' + r.headers.get('content-type'))`), '200 image/png');
  assert.equal(await page.eval(`document.querySelector('meta[name="twitter:card"]').content`), 'summary_large_image');
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
  test(`every player control is reachable at ${width}px`, () => with_page({ width, fake_mic: true }, async (page) => {
    await sleep(300);
    const clipped = await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')]
      .filter((b) => { const r = b.getBoundingClientRect(); return r.width === 0 || r.left < 0 || r.right > innerWidth; })
      .map((b) => b.textContent)`);
    assert.deepEqual(clipped, []);
    // With the mic on, the readout, note lane and Tuning score all stay inside the player card (it clips)
    await page.eval(`[...document.querySelectorAll('.pitch-meter button')].find((b) => b.textContent.startsWith('Mic')).click()`);
    await page.wait_for(`!!document.querySelector('.pitch-meter .tuning-score')`);
    const outside = await page.eval(`(() => {
      const card = document.querySelector('.player-container').getBoundingClientRect();
      // Boxes, not wrappers: a solo voice row is display: contents (no box of its own)
      return [...document.querySelectorAll('.pitch-meter > *, .pitch-meter .voice-row > *')]
        .filter((e) => getComputedStyle(e).display !== 'contents')
        .filter((e) => { const r = e.getBoundingClientRect(); return r.left < card.left || r.right > card.right; })
        .map((e) => e.className);
    })()`);
    assert.deepEqual(outside, [], 'mic meter fits the player');
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
  // The app shell mounts before the player renders its song: wait for it rather than read once
  await page.wait_for(`${now_title} === ${JSON.stringify(inserted)}`);
  assert.deepEqual(await page.eval(queue_titles), [...start, queued]);
}));

test('first visit, autoplay blocked: the clock waits, Play shows, Space starts the song', async () => {
  const real = await launch({ real_autoplay: true });
  const page = await real.new_page({});
  try {
    await page.goto();
    const time = `document.querySelector('.current-time')?.textContent`;
    const start = await page.eval(time);
    const button = `[...document.querySelectorAll('.player-main-controls-row button')].find((b) => /Playback/.test(b.title))?.textContent`;
    await page.wait_for(`${button} === 'Play'`, 6000);
    assert.equal(await page.eval(time), start, 'time did not run while the video was not playing');
    assert.equal(await page.eval(`window.KtvSync.debug().karaoke_state`), -1, 'YouTube: unstarted');
    await page.eval(`document.activeElement?.blur()`);
    await page.key(' '); // a trusted key press is the user gesture autoplay needs
    await page.wait_for(`${button} === 'Pause'`);
    // YouTube may refuse to play for a datacenter IP (CI); either way the time moves only while it plays
    const state = `window.KtvSync.debug().karaoke_state`;
    await page.wait_for(`${state} === 1 || ${button} === 'Play'`, 10000);
    if (await page.eval(`${state} === 1`)) {
      await page.wait_for(`${time} !== ${JSON.stringify(start)}`);
    } else {
      assert.equal(await page.eval(time), start, 'YouTube did not start the video: the time still waits');
    }
    assert.deepEqual(page.errors, [], 'console errors');
  } finally {
    await page.close();
    await real.close();
  }
});

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

test('mic: room check, noise gate, held notes give a Tuning score, a phrase score and a note lane, the finished song shows a result card, named for the party leaderboard, and joins Recent scores', () => with_page({ fake_mic: true }, async (page) => {
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
  await page.wait_for(`/^\\d+$/.test(document.querySelector('.tuning-score .tuning-value')?.textContent ?? '')`);
  // A breath ends the phrase: it gets its own score, and the held notes sit in the lane on grid lines
  await page.eval(`window.__mic_gain.gain.value = 0`);
  await page.wait_for(`/^\\d+$/.test(document.querySelector('.note-lane-phrase .tuning-value')?.textContent ?? '')`, 3000);
  const lane = await page.eval(`JSON.stringify({
    phrase: Number(document.querySelector('.note-lane-phrase .tuning-value').textContent),
    notes: document.querySelectorAll('.note-lane-plot .lane-note').length,
    good: document.querySelectorAll('.note-lane-plot .lane-note.good').length,
  })`);
  const { phrase, notes, good } = JSON.parse(lane);
  assert.ok(phrase >= 90, `in-tune phrase scores high, got ${phrase}`);
  assert.ok(notes >= 4 && good === notes, `held notes drawn in tune: ${good}/${notes}`);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`!!document.querySelector('.score-card')`);
  const card = await page.eval(`JSON.stringify({
    value: document.querySelector('.score-card-value').textContent,
    song: document.querySelector('.score-card-song').textContent,
  })`);
  const { value, song } = JSON.parse(card);
  assert.ok(Number(value) >= 90, `sawtooth in tune scores high, got ${value}`);
  assert.ok(song.startsWith(sung), `card names the finished song: ${song}`);
  // Name the take for the party leaderboard (typing in the field does not type into the song search)
  await page.eval(`document.getElementById('singer_name').focus()`);
  for (const c of '  Pim ') await page.key(c);
  await page.key('Enter');
  await page.wait_for(`document.querySelector('.score-card-singer strong')?.textContent === 'Pim'`);
  assert.equal(await page.eval(search_value), '');
  await page.reload();
  const board = await page.eval(`(async () => {
    [...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click();
    await new Promise((r) => setTimeout(r, 200));
    return JSON.stringify({
      leaders: [...document.querySelectorAll('.leaderboard .score-row-song strong')].map((e) => e.textContent),
      recent: [...document.querySelectorAll('.score-history > .score-row .score-row-song')].map((e) => e.textContent),
    });
  })()`);
  assert.deepEqual(JSON.parse(board), { leaders: ['Pim'], recent: [`${sung} · Pim`] }, 'history and name survive reload');
}));

test('duet: two mics on one stereo receiver each get a pitch, lane and Tuning score, and each take gets its own result card and name', () => with_page({ fake_mic: 'stereo' }, async (page) => {
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Settings')).click()`);
  await page.wait_for(`!!document.getElementById('duet_toggle')`);
  await page.click('#duet_toggle');
  await page.wait_for(`document.getElementById('duet_toggle').textContent === 'ON'`);
  await page.eval(`[...document.querySelectorAll('.pitch-meter button')].find((b) => b.textContent.startsWith('Mic')).click()`);
  await page.wait_for(`!!window.__mic2_gain`);
  await page.wait_for(`document.querySelectorAll('.voice-row.duet').length === 2`);
  await page.wait_for(`[...document.querySelectorAll('.pitch-note')].every((n) => n.textContent !== 'Room check…')`, 6000);
  // Left sings A3 then B3, right sings C4 then D4, both in tune
  await page.eval(`window.__mic_gain.gain.value = 0.3; window.__mic2_gain.gain.value = 0.3`);
  await page.wait_for(`JSON.stringify([...document.querySelectorAll('.pitch-note')].map((n) => n.textContent)) === '["A3","C4"]'`);
  for (const [left, right] of [[246.94, 293.66], [220, 261.63], [246.94, 293.66], [220, 261.63]]) {
    await sleep(500);
    await page.eval(`window.__mic.frequency.value = ${left}; window.__mic2.frequency.value = ${right}`);
  }
  await page.wait_for(`[...document.querySelectorAll('.voice-row.duet .tuning-score .tuning-value')].every((v) => /^\\d+$/.test(v.textContent))`, 6000);
  assert.equal(await page.eval(`document.querySelectorAll('.voice-row.duet .tuning-score').length`), 2);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`document.querySelectorAll('.score-card').length === 2`);
  const parts = await page.eval(`JSON.stringify([...document.querySelectorAll('.score-card-part')].map((e) => e.textContent))`);
  assert.deepEqual(JSON.parse(parts), ['Duet · singer 1 (left mic)', 'Duet · singer 2 (right mic)']);
  // Each card names its own singer
  for (const [id, name] of [['singer_name', 'Pim'], ['singer_name_2', 'Ton']]) {
    await page.eval(`document.getElementById('${id}').focus()`);
    for (const c of name) await page.key(c);
    await page.key('Enter');
  }
  await page.wait_for(`JSON.stringify([...document.querySelectorAll('.score-card-singer strong')].map((e) => e.textContent)) === '["Pim","Ton"]'`);
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click()`);
  await page.wait_for(`document.querySelectorAll('.leaderboard .score-row-song strong').length === 2`);
}));

test('melody score: with a melody saved on this device, the lane draws the tune in the singer\'s octave, the right note in another octave scores high, a wrong note pulls it down, and the result card shows it', () => with_page({ fake_mic: true }, async (page) => {
  const melody_value = `document.querySelector('.melody-score .tuning-value')?.textContent ?? ''`;
  await page.wait_for(`!!localStorage.getItem('ktv.session.v1')`);
  assert.equal(await page.eval(`!!document.querySelector('.melody-score')`), false, 'no melody data, no melody score');
  // The tune: A3 through the whole song, as 10 s notes (a note is at most 30 s; no melody ships, plan 002 item 7)
  await page.eval(`(() => {
    const id = JSON.parse(localStorage.getItem('ktv.session.v1')).current.song.id;
    const notes = Array.from({ length: 90 }, (_, i) => [i * 10, i * 10 + 10, 57]);
    localStorage.setItem('ktv.melodies.v1', JSON.stringify({ [id]: notes }));
  })()`);
  await page.reload();
  // The melody score skips frames while the video clock stands still, and CI runners often get no YouTube playback:
  // a running clock for the scorer, so this checks scoring, not YouTube
  await page.wait_for(`typeof window.KtvSync?.time === 'function'`);
  await page.eval(`(() => { const t0 = performance.now(); window.KtvSync.time = () => 30 + (performance.now() - t0) / 1000; })()`);
  await page.eval(`[...document.querySelectorAll('.pitch-meter button')].find((b) => b.textContent.startsWith('Mic')).click()`);
  await page.wait_for(`!!window.__mic_gain`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent !== 'Room check…'`, 6000);
  await page.wait_for(`!!document.querySelector('.melody-score')`);
  // A4: the right note an octave up
  await page.eval(`window.__mic.frequency.value = 440; window.__mic_gain.gain.value = 0.3`);
  // The room check's silent second counts as missed tune, so the score climbs while the right note is held
  await page.wait_for(`Number(${melody_value}) >= 90`, 12000);
  const right = Number(await page.eval(melody_value));
  // A held note is drawn once it ends: take a breath
  await page.eval(`window.__mic_gain.gain.value = 0`);
  await page.wait_for(`!!document.querySelector('.note-lane-plot .lane-note')`, 3000);
  await page.eval(`window.__mic_gain.gain.value = 0.3`);
  const lane = JSON.parse(await page.eval(`JSON.stringify({
    targets: document.querySelectorAll('.note-lane-plot .lane-target').length,
    now: !!document.querySelector('.note-lane-plot .lane-now'),
    target_y: document.querySelector('.note-lane-plot .lane-target')?.getAttribute('y'),
    bar_y: document.querySelector('.note-lane-plot .lane-note')?.getAttribute('y'),
  })`));
  assert.ok(lane.targets >= 1 && lane.now, `tune drawn with a now marker: ${JSON.stringify(lane)}`);
  // Target (height 0.9 row) and sung bar (0.6 row) centred on the same row: the tune moved to the singer's octave
  assert.ok(Math.abs(Number(lane.target_y) - Number(lane.bar_y)) < 10, `same row: ${JSON.stringify(lane)}`);
  // B4: a whole tone off
  await page.eval(`window.__mic.frequency.value = 493.88`);
  await page.wait_for(`Number(${melody_value}) < ${right} - 20`, 10000);
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`!!document.querySelector('.score-card-melody')`);
  assert.match(await page.eval(`document.querySelector('.score-card-melody').textContent`), /^Melody \d+ · right notes, any octave$/);
}));

test('phone remote: the Keypad QR opens /remote, a phone sees the stage, queues by code, playback buttons only when the host allows, a new link sends old phones away', () => with_page({}, async (booth) => {
  const status = `document.querySelector('.phone-remote [role=status]')?.textContent ?? ''`;
  await booth.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Keypad')).click()`);
  await booth.wait_for(`!!document.querySelector('.phone-remote .toggle-btn')`);
  assert.match(await booth.eval(status), /^Off/, 'off until the host turns it on');
  await booth.click('.phone-remote .toggle-btn');
  await booth.wait_for(`${status}.startsWith('Online')`);
  const url = await booth.eval(`document.querySelector('.phone-remote-qr-link').href`);
  assert.match(url, /\/remote#r=[A-Za-z0-9_-]{22}$/);
  const on_stage = await booth.eval(now_title);
  const phone = await browser.new_page({ width: 390, height: 844 });
  try {
    await phone.send('Page.navigate', { url });
    await phone.wait_for(`document.getElementById('now_title')?.textContent === ${JSON.stringify(on_stage)}`);
    await phone.wait_for(`document.getElementById('status').textContent === 'Connected to the booth'`);
    await booth.wait_for(`${status}.includes('1 phone connected')`);
    assert.equal(await phone.eval(`getComputedStyle(document.getElementById('controls')).display`), 'none', 'queue only by default');
    assert.ok(await phone.eval(`document.querySelectorAll('#next li').length > 0`), 'up next listed');
    // Queue by code: the booth adds it and answers the phone
    await phone.eval(`(() => { const c = document.getElementById('code'); c.value = '10001'; c.dispatchEvent(new Event('input')); document.getElementById('queue').click(); })()`);
    await phone.wait_for(`document.getElementById('reply').textContent.startsWith('Queued 10001')`);
    assert.equal(await phone.eval(`document.getElementById('reply').dataset.ok`), 'true');
    await booth.wait_for(`document.querySelector('.auto-dj-toast')?.textContent.includes('📱 Queued 10001')`);
    await phone.wait_for(`[...document.querySelectorAll('#next li')].some((li) => li.textContent.startsWith('10001') && li.textContent.includes('📱 Phone'))`);
    // A code the booth does not have is refused with a reason
    await phone.eval(`(() => { const c = document.getElementById('code'); c.value = '00000'; c.dispatchEvent(new Event('input')); document.getElementById('queue').click(); })()`);
    await phone.wait_for(`document.getElementById('reply').textContent === 'No song with code 00000'`);
    // The host allows playback: Skip on the phone moves the booth on
    await booth.click('#remote_allow_playback');
    await phone.wait_for(`getComputedStyle(document.getElementById('controls')).display === 'grid'`);
    await phone.click('#controls button[data-cmd=skip]');
    await booth.wait_for(`${now_title} !== ${JSON.stringify(on_stage)}`);
    const next_up = await booth.eval(now_title);
    await phone.wait_for(`document.getElementById('now_title').textContent === ${JSON.stringify(next_up)}`);
    // New link (two taps): the old phone is told to scan again, and the QR changes
    await booth.click('.phone-remote-options .action-btn');
    await booth.click('.phone-remote-options .action-btn');
    await phone.wait_for(`document.getElementById('status').textContent.startsWith('The booth made a new code')`);
    await booth.wait_for(`document.querySelector('.phone-remote-qr-link').href !== ${JSON.stringify(url)}`);
    await booth.wait_for(`${status}.startsWith('Online')`);
    assert.deepEqual(await phone.eval('window.__csp'), [], 'phone CSP violations');
    assert.deepEqual(phone.errors, [], 'phone console errors');
  } finally {
    await phone.close();
  }
}));

test('search: romanised alias and wrong keyboard layout both find Thai songs', () => with_page({}, async (page) => {
  const titles = `[...document.querySelectorAll('.song-title')].map((e) => e.textContent)`;
  for (const key of 'rak mai wai'.replace(/ /g, '')) await page.key(key);
  // Curated songs list first; the full library may add more "Rak Mai Wai..." titles after them
  await page.wait_for(`${titles}[0] === 'รักไม่ไหวแล้วโว้ย'`);
  await page.key('Escape');
  // "รัก" typed with the keyboard left on English
  for (const key of 'iyd') await page.key(key);
  await page.wait_for(`!!document.querySelector('.retyped-text')`);
  assert.equal(await page.eval(`document.querySelector('.retyped-text strong').textContent`), 'รัก');
  // Title or artist (library songs match on either)
  const cards = `[...document.querySelectorAll('.song-details')].map((e) => e.textContent)`;
  assert.ok(await page.eval(`${cards}.length > 0 && ${cards}.every((t) => t.includes('รัก'))`));
}));

test('full library: joins after first paint, capped list grows, alias search, a library song plays', () => with_page({}, async (page) => {
  const total = `parseInt(document.querySelector('.catalog-meta-row .count-text').textContent.replace(/\\D/g, ''), 10)`;
  const cards = `document.querySelectorAll('.song-card').length`;
  await page.wait_for(`${total} > 5000`);
  assert.equal(await page.eval(cards), 60, 'only the first page of cards is rendered');
  await page.eval(`document.querySelector('.catalog-more-row button').click()`);
  await page.wait_for(`${cards} === 120`);
  // GMM's romanised title of กำก่าสกาฬสินธ์ุ
  for (const key of 'kamkasakalasin') await page.key(key);
  const card = `[...document.querySelectorAll('.song-card')].find((c) => c.querySelector('.song-title').textContent === 'กำก่าสกาฬสินธ์ุ')`;
  await page.wait_for(`!!${card}`);
  await page.eval(`${card}.querySelector('.play-now').click()`);
  await page.wait_for(`document.querySelector('.sr-only[role=status]').textContent.includes('กำก่าสกาฬสินธ์ุ')`);
}));

test('Add URL: a video already in the songbook is queued as that song, not copied', () => with_page({}, async (page) => {
  await page.wait_for(`document.querySelector('.catalog-meta-row .count-text')?.textContent.replace(/\\D/g, '') > 5000`);
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Add URL').click()`);
  await page.wait_for(`!!document.getElementById('custom_url')`);
  await page.eval(`(() => {
    const input = document.getElementById('custom_url');
    input.value = 'https://youtu.be/ow7Ebs5VJjo';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  })()`);
  await sleep(100);
  await page.eval(`[...document.querySelectorAll('.submit-btn')].find((b) => b.textContent === 'Add to Queue').click()`);
  await page.wait_for(`document.querySelector('.form-feedback')?.textContent === 'Already in the songbook: กำก่าสกาฬสินธ์ุ, keypad code 31008'`);
  await page.wait_for(`JSON.parse(localStorage.getItem('ktv.session.v1')).queue.some((it) => it.song.id === 'yt_ow7Ebs5VJjo')`);
  assert.deepEqual(await page.eval(`JSON.parse(localStorage.getItem('ktv.session.v1')).custom_songs`), [], 'no custom copy');
}));

test('fullscreen: typing lists songs beside the video, and one plays from there', () => with_page({ width: 1280, height: 800 }, async (page) => {
  await page.wait_for(`parseInt(document.querySelector('.catalog-meta-row .count-text')?.textContent.replace(/\\D/g, ''), 10) > 5000`);
  await page.tap('.player-main-controls-row button[title="Toggle Fullscreen Cinema Mode"]');
  await page.wait_for(`document.fullscreenElement?.classList.contains('stage-player-side')`);
  assert.equal(await page.eval(`!!document.querySelector('.quick-search')`), false, 'no panel before typing');
  for (const key of 'kamkasakalasin') await page.key(key);
  const panel = `document.querySelector('.stage-player-side:fullscreen > .quick-search')`;
  await page.wait_for(`${panel} && getComputedStyle(${panel}).display === 'flex'`);
  // Beside the video, not over it
  const layout = JSON.parse(await page.eval(`JSON.stringify({ video: document.querySelector('.video-stage').getBoundingClientRect(), panel: ${panel}.getBoundingClientRect() })`));
  assert.ok(layout.video.right <= layout.panel.left + 1, `video ${layout.video.right} overlaps panel ${layout.panel.left}`);
  if (process.env.KTV_SHOT) await page.screenshot(process.env.KTV_SHOT);
  const item = `[...document.querySelectorAll('.quick-search-item')].find((c) => c.querySelector('.quick-search-title').textContent === 'กำก่าสกาฬสินธ์ุ')`;
  await page.wait_for(`!!${item}`);
  await page.eval(`${item}.querySelector('.play-now').click()`);
  await page.wait_for(`document.querySelector('.now-title').textContent === 'กำก่าสกาฬสินธ์ุ'`);
  await page.eval(`document.querySelector('.quick-search .clear-search-btn').click()`);
  await page.wait_for(`!document.querySelector('.quick-search')`);
  assert.ok(await page.eval(`!!document.fullscreenElement`), 'still fullscreen');
}));

test('library MV: timing tools offer the found official video; the timed guide sticks to the song', () => with_page({}, async (page) => {
  const loaded = `parseInt(document.querySelector('.catalog-meta-row .count-text')?.textContent.replace(/\\D/g, ''), 10) > 5000`;
  await page.wait_for(`localStorage.getItem('ktv.settings.v1')`);
  await page.eval(`(() => { const s = JSON.parse(localStorage.getItem('ktv.settings.v1')); s.show_timing_tools = true; localStorage.setItem('ktv.settings.v1', JSON.stringify(s)); })()`);
  await page.reload();
  await page.wait_for(loaded);
  // คนบ้านเดียวกัน - ไผ่ พงศธร (GMM Karaoke 37849): suggested official MV wv-G-kM9pUs, not timed yet
  const play_code = async () => {
    await page.eval(`document.activeElement?.blur()`);
    for (const key of '37849') await page.key(key);
    await page.wait_for(`document.querySelector('.song-card .song-code-tag')?.textContent === '#37849'`);
    await page.eval(`document.querySelector('.song-card .play-now').click()`);
    await page.wait_for(`document.querySelector('.now-title').textContent === 'คนบ้านเดียวกัน'`);
  };
  const vocal = `[...document.querySelectorAll('.player-main-controls-row button')].some((b) => b.textContent.startsWith('Vocal:'))`;
  await play_code();
  assert.equal(await page.eval(vocal), false, 'an unchecked suggestion gives no Vocal button');
  await page.wait_for(`!!document.querySelector('.timing-suggestion a[href$="wv-G-kM9pUs"]')`);
  await page.eval(`document.querySelector('.timing-suggestion button').click()`);
  await page.wait_for(`document.querySelector('.timing-status')?.textContent === 'saved on this device' && ${vocal}`);
  assert.equal(await page.eval(`JSON.parse(localStorage.getItem('ktv.guides.v1'))['yt_-x4Urp1Qrrk'].video_id`), 'wv-G-kM9pUs');
  // Another song, then this one again from the list: the timing saved on this device comes with it
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`document.querySelector('.now-title').textContent !== 'คนบ้านเดียวกัน'`);
  await page.eval(`document.querySelector('.clear-search-btn')?.click()`);
  await play_code();
  await page.wait_for(vocal);
}));

test('practice: A then B loops the part, a new song clears it; Chords opens a web search for the song', () => with_page({}, async (page) => {
  const buttons = `[...document.querySelectorAll('.practice-row button')]`;
  const press = (label) => page.eval(`${buttons}.find((b) => b.textContent === ${JSON.stringify(label)}).click()`);
  assert.equal(await page.eval(`${buttons}.find((b) => b.textContent === 'B').disabled`), true, 'B waits for A');
  await press('A');
  // Move 10 s on without depending on YouTube playing (CI may not)
  await page.eval(`[...document.querySelectorAll('.player-quick-controls button')].find((b) => b.textContent === '+10s').click()`);
  await sleep(100);
  await press('B');
  await page.wait_for(`document.querySelector('.practice-label').textContent.startsWith('Looping')`);
  const loop = JSON.parse(await page.eval(`JSON.stringify(window.KtvSync.debug().loop)`));
  assert.ok(loop && loop[1] - loop[0] >= 9, `loop ${loop}`);
  const title = await page.eval(`document.querySelector('.now-title').textContent`);
  const href = await page.eval(`document.querySelector('.practice-chords').href`);
  assert.equal(decodeURIComponent(new URL(href).searchParams.get('q')).startsWith(`คอร์ด ${title} `), true, href);
  assert.equal(await page.eval(`document.querySelector('.practice-chords').rel`), 'noopener noreferrer');
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`window.KtvSync.debug().loop === null && document.querySelector('.practice-label').textContent.startsWith('Loop a part')`);
}));

test('count-in: tapping the beat sets the tempo (saved on this device), Count-in seeks 4 beats before the loop\'s A and runs until the part begins', () => with_page({}, async (page) => {
  // A controllable clock and a recording Replay (YouTube playback is not needed, and CI often has none)
  const install_clock = `(() => {
    window.__clock = { base: 30, at: null };
    window.__restarts = [];
    window.KtvSync.time = () => window.__clock.at === null ? window.__clock.base : window.__clock.base + (performance.now() - window.__clock.at) / 1000;
    window.KtvSync.restart = (t) => { window.__restarts.push(t); window.__clock = { base: t, at: performance.now() }; };
  })()`;
  const label = `document.querySelector('.tempo-label')?.textContent`;
  await page.wait_for(`typeof window.KtvSync?.time === 'function'`);
  await page.eval(install_clock);
  assert.equal(await page.eval(label), 'Tap the beat');
  assert.equal(await page.eval(`[...document.querySelectorAll('.tempo-tools button')].find((b) => b.textContent === 'Count-in').disabled`), true);
  for (const [i, t] of [30, 30.5, 31, 31.5, 32].entries()) {
    await page.eval(`window.__clock.base = ${t}`);
    await page.click('.tempo-tap');
    if (i < 3) await page.wait_for(`${label} === 'Tap ${3 - i} more'`);
  }
  await page.wait_for(`${label} === '♩ 120'`);
  await page.reload();
  await page.wait_for(`${label} === '♩ 120'`);
  await page.eval(install_clock);
  // Loop start A a little after the 40.0 beat: the part lands on 40.0, clicks at 38-39.5, playback from 37.0
  await page.eval(`window.__clock.base = 40.1`);
  await page.eval(`[...document.querySelectorAll('.practice-row .practice-btn')].find((b) => b.textContent === 'A').click()`);
  await page.eval(`window.__clock.base = 45`);
  await page.eval(`[...document.querySelectorAll('.tempo-tools button')].find((b) => b.textContent === 'Count-in').click()`);
  await page.wait_for(`window.__restarts.length === 1`);
  assert.equal(await page.eval(`window.__restarts[0]`), 37);
  const count_in = `[...document.querySelectorAll('.tempo-tools button')].find((b) => b.textContent === 'Count-in')`;
  await page.wait_for(`${count_in}.disabled`, 2000);
  await page.wait_for(`!${count_in}.disabled`, 6000);
  assert.ok(await page.eval(`window.KtvSync.time() >= 40`), 'busy until the part began');
}));

test('chords by ear: typed and tapped changes show as the song plays, transpose for display, saved on this device', () => with_page({}, async (page) => {
  const buttons = `[...document.querySelectorAll('.practice-row button')]`;
  const press = (label) => page.eval(`${buttons}.find((b) => b.textContent === ${JSON.stringify(label)}).click()`);
  const now_chord = `document.querySelector('.chord-now-name')?.textContent`;
  const notice = `document.querySelector('.chord-editor .timing-notice')?.textContent`;
  const type = async (text) => {
    await page.eval(`document.getElementById('chord_name').focus()`);
    for (const c of text) await page.key(c);
    await page.key('Enter');
  };
  await press('Chords by ear');
  await page.wait_for(`!!document.getElementById('chord_name')`);
  await type('Hm');
  await page.wait_for(`${notice}?.startsWith('A chord starts with a note')`);
  await page.eval(`document.getElementById('chord_name').value = ''; document.getElementById('chord_name').dispatchEvent(new Event('input', { bubbles: true }))`);
  await type('am');
  await page.wait_for(`${now_chord} === 'Am'`);
  assert.equal(await page.eval(search_value), '', 'typing a chord does not type into the song search');
  // The next change 10 s on, tapped from the palette of chords used so far
  await page.eval(`[...document.querySelectorAll('.player-quick-controls button')].find((b) => b.textContent === '-10s').click()`);
  await sleep(300);
  await page.eval(`document.querySelector('.chord-palette button').click()`);
  await page.wait_for(`document.querySelectorAll('.chord-marks li').length === 2`);
  await page.eval(`[...document.querySelectorAll('.chord-shift button')][1].click()`);
  await page.wait_for(`${now_chord} === 'A#m' && document.querySelector('.chord-shift-label').textContent === 'Key: +1'`);
  const saved = JSON.parse(await page.eval(`localStorage.getItem('ktv.chords.v1')`));
  const [chart] = Object.values(saved);
  assert.deepEqual(chart.map((m) => m.chord), ['Am', 'Am'], 'stored as entered, not transposed');
  await page.reload();
  await page.wait_for(`${now_chord} === 'Am'`);
  await press('Edit chords');
  await page.wait_for(`document.querySelectorAll('.chord-mark-remove').length === 2`);
  await page.eval(`document.querySelector('.chord-mark-remove').click()`);
  await page.eval(`document.querySelector('.chord-mark-remove').click()`);
  await page.wait_for(`!document.querySelector('.chord-now') && localStorage.getItem('ktv.chords.v1') === '{}'`);
}));

test('search: a typo still finds the song, and says it is showing close matches', () => with_page({}, async (page) => {
  await page.wait_for(`parseInt(document.querySelector('.catalog-meta-row .count-text')?.textContent.replace(/\\D/g, ''), 10) > 5000`);
  await page.eval(`document.activeElement?.blur()`);
  for (const key of 'bodyslan') await page.key(key);
  await page.wait_for(`document.querySelector('.catalog-meta-row [role=status]')?.textContent.startsWith('No exact match')`);
  assert.ok(await page.eval(`[...document.querySelectorAll('.song-artist')].some((a) => /bodyslam/i.test(a.textContent))`));
}));

test('auto-timed vocal: an official-audio song has Vocal at once, and ±0.5 s fixes it on this device', () => with_page({}, async (page) => {
  await page.wait_for(`parseInt(document.querySelector('.catalog-meta-row .count-text')?.textContent.replace(/\\D/g, ''), 10) > 5000`);
  // ผิดตรงไหน - เบิร์ด ธงไชย (GMM Karaoke 34201): official audio StGwcpcGP7E at -18.2 s
  await page.eval(`document.activeElement?.blur()`);
  for (const key of '34201') await page.key(key);
  await page.wait_for(`document.querySelector('.song-card .song-code-tag')?.textContent === '#34201'`);
  await page.eval(`document.querySelector('.song-card .play-now').click()`);
  await page.wait_for(`document.querySelector('.now-title').textContent === 'ผิดตรงไหน'`);
  const vocal = `[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent.startsWith('Vocal:'))`;
  await page.wait_for(`!!${vocal}`);
  await page.eval(`${vocal}.click()`);
  await page.wait_for(`document.querySelector('.vocal-nudge-label')?.textContent === 'Auto-timed. Out of step?'`);
  await page.eval(`[...document.querySelectorAll('.vocal-nudge button')].find((b) => b.textContent === 'Vocal behind').click()`);
  await page.wait_for(`JSON.parse(localStorage.getItem('ktv.guides.v1') ?? '{}')['yt_4W_T9DPufmQ']?.offset_secs === -17.7`);
  await page.wait_for(`document.querySelector('.vocal-nudge-label')?.textContent === 'Out of step?'`);
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

test('keyboard: a keyboard-focused button takes Space; after a mouse click Space stays the pause key', () => with_page({}, async (page) => {
  const card = (code) => `[...document.querySelectorAll('.song-card')].find((c) => c.querySelector('.song-code-tag').textContent === '#${code}')`;
  // Keyboard user: focus lands on Play without a pointer press, Space plays that song
  const title = await page.eval(`(() => { const c = ${card('10005')}; c.querySelector('.play-now').focus(); return c.querySelector('.song-title').textContent; })()`);
  await page.key(' ');
  await page.wait_for(`${now_title} === ${JSON.stringify(title)}`);
  await page.wait_for(`document.querySelector('.sr-only[role=status]').textContent.startsWith('Now singing: ' + ${JSON.stringify(title)})`);
  // Mouse user: a real click on ☆, then Space must not click it again (it would un-star the song)
  // Scrolled into view first: a real click lands at on-screen coordinates
  const rect = await page.eval(`(() => { const b = ${card('10001')}.querySelector('.fav-btn'); b.scrollIntoView({ block: 'center' }); return JSON.stringify(b.getBoundingClientRect()); })()`);
  const { x, y, width, height } = JSON.parse(rect);
  for (const type of ['mousePressed', 'mouseReleased']) {
    await page.send('Input.dispatchMouseEvent', { type, x: x + width / 2, y: y + height / 2, button: 'left', clickCount: 1 });
  }
  await page.wait_for(`${card('10001')}.querySelector('.fav-btn').classList.contains('on')`);
  await page.key(' ');
  await sleep(300);
  assert.ok(await page.eval(`${card('10001')}.querySelector('.fav-btn').classList.contains('on')`), 'still starred');
}));

test('booth input: a typed code + Enter queues it, an unknown code says so; a game controller pauses and skips', () => with_page({}, async (page) => {
  await page.eval(`document.activeElement?.blur()`);
  const start = await page.eval(queue_titles);
  for (const c of '10001') await page.key(c);
  await page.wait_for(`${search_value} === '10001'`);
  await page.key('Enter');
  await page.wait_for(`${search_value} === ''`);
  await page.wait_for(`document.querySelector('.auto-dj-toast')?.textContent.includes('Queued 10001')`);
  assert.deepEqual(await page.eval(queue_titles), [...start, 'รักแล้วยอมได้']);
  await page.eval(`document.activeElement?.blur()`);
  for (const c of '89999') await page.key(c);
  await page.key('Enter');
  await page.wait_for(`document.querySelector('.auto-dj-toast')?.textContent.includes('No song with code 89999')`);
  assert.equal(await page.eval(search_value), '89999', 'kept so it can be corrected');
  await page.key('Escape');

  // A standard-mapping controller (the Gamepad API cannot be driven from CDP, so the page's pad list is stubbed)
  // Browsers poll pads (and run animation frames) only in the visible tab
  await page.send('Page.bringToFront');
  await page.wait_for(`document.visibilityState === 'visible'`);
  await page.eval(`(() => {
    const buttons = Array.from({ length: 17 }, () => ({ pressed: false }));
    window.__pad = { index: 0, connected: true, mapping: 'standard', buttons };
    window.__polls = 0;
    navigator.getGamepads = () => { window.__polls += 1; return [window.__pad]; };
    window.dispatchEvent(new Event('gamepadconnected'));
  })()`);
  // A button already down on the first poll counts as held, not pressed: start pressing once polling runs
  await page.wait_for(`window.__polls > 1`);
  const press = async (i) => {
    await page.eval(`window.__pad.buttons[${i}].pressed = true`);
    await sleep(100);
    await page.eval(`window.__pad.buttons[${i}].pressed = false`);
    await sleep(100);
  };
  const paused = `window.KtvSync.debug().paused`;
  const was_paused = await page.eval(paused);
  await press(0);
  await page.wait_for(`${paused} === ${!was_paused}`);
  const before = await page.eval(now_title);
  await press(9);
  await page.wait_for(`${now_title} !== ${JSON.stringify(before)}`);
}));

test('privacy: the note is served, and Clear my data (two taps) resets this device', () => with_page({}, async (page) => {
  assert.match(await page.eval(`fetch('/privacy.html').then((r) => r.status + ' ' + r.headers.get('content-type'))`), /^200 text\/html/);
  await page.eval(`document.querySelectorAll('.fav-btn')[0].click()`);
  await page.eval(`[...document.querySelectorAll('.shortcut-help button')].find((b) => b.textContent === 'Close').click()`);
  await page.wait_for(`JSON.parse(localStorage.getItem('ktv.picks.v1') || '{}').favourites?.length === 1`);
  await page.eval(`localStorage.setItem('_ktv_seek_lead', '0.25'); localStorage.setItem('other.site', 'keep')`);
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Settings').click()`);
  const clear = `[...document.querySelectorAll('.clear-data-row button')]`;
  await page.wait_for(`${clear}.length === 1`);
  await page.eval(`${clear}[0].click()`);
  await page.wait_for(`${clear}[0].textContent === 'Tap again to clear everything'`);
  assert.ok(await page.eval(`!!localStorage.getItem('ktv.picks.v1')`), 'first tap only asks');
  await page.eval(`${clear}[0].click()`);
  await sleep(300);
  await page.wait_for(`!!document.querySelector('.ktv-app-wrapper')`);
  await page.wait_for(`!!document.querySelector('.shortcut-help')`);
  assert.equal(await page.eval(`document.querySelectorAll('.fav-btn.on').length`), 0, 'favourites gone');
  assert.equal(await page.eval(`localStorage.getItem('_ktv_seek_lead')`), null);
  assert.equal(await page.eval(`localStorage.getItem('other.site')`), 'keep', 'other keys untouched');
}));

test('installable: manifest parses, icons load, Chrome reports no installability errors', () => with_page({}, async (page) => {
  const manifest = await page.send('Page.getAppManifest');
  assert.deepEqual(manifest.errors, [], 'manifest parse errors');
  const parsed = JSON.parse(manifest.data);
  assert.equal(parsed.display, 'standalone');
  for (const icon of parsed.icons) {
    assert.equal(await page.eval(`fetch(${JSON.stringify(icon.src)}).then((r) => r.status + ' ' + r.headers.get('content-type'))`), '200 image/png', icon.src);
  }
  const { installabilityErrors } = await page.send('Page.getInstallabilityErrors');
  // Each test runs in a fresh (incognito-like) browser context, where Chrome never offers install
  assert.deepEqual(installabilityErrors.map((e) => e.errorId).filter((id) => id !== 'in-incognito'), []);
}));

test('desktop: player and songbook side by side in one screen (no page scroll)', () => with_page({ width: 1280, height: 800 }, async (page) => {
  assert.equal(await page.eval(`getComputedStyle(document.querySelector('.ktv-split-stage')).display`), 'grid');
  assert.equal(await page.eval(`document.documentElement.scrollHeight <= innerHeight`), true);
  const [player, songbook] = JSON.parse(await page.eval(`JSON.stringify(['.stage-player-side', '.stage-control-side'].map((s) => document.querySelector(s).getBoundingClientRect().left))`));
  assert.ok(songbook > player + 400, 'songbook column to the right of the player');
}));

test('TV mode: toggle is remembered, shows Up next, still fits one 1080p screen', () => with_page({ width: 1920, height: 1080 }, async (page) => {
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'TV').click()`);
  await page.wait_for(`document.querySelector('.ktv-app-wrapper').classList.contains('tv-mode')`);
  assert.equal(await page.eval(`document.querySelectorAll('.up-next-item').length`), 3);
  assert.equal(await page.eval(`document.documentElement.scrollHeight <= innerHeight`), true, 'fits the screen');
  assert.equal(await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].filter((b) => b.getBoundingClientRect().bottom > innerHeight).length`), 0, 'controls on screen');
  await page.reload();
  await page.wait_for(`document.querySelector('.ktv-app-wrapper').classList.contains('tv-mode')`);
}));

// A real devnet USDC transfer (0.001 USDC to `tip_wallet`, with a memo). The mock RPC replays it for each "payment":
// its first account key becomes the paid reference, and the memo and amount are set per payment.
const tip_tx = JSON.parse(readFileSync(new URL('../fixtures/tip/devnet_usdc_transfer_with_memo.json', import.meta.url)));
const tip_wallet = '75AjMdh7Gn1TLigfze541AVJGJ4TyqBEaRZk3pozfBza';

/** Mock devnet RPC: nothing is paid until `pay(reference, { signature, memo, amount })`. `calls`: methods asked. */
async function mock_devnet(page) {
  const payments = new Map(); // signature -> payment
  const calls = [];
  await page.mock_json('https://api.devnet.solana.com*', (request) => {
    const call = JSON.parse(request.postData);
    calls.push(call.method);
    switch (call.method) {
      case 'getSignaturesForAddress': {
        const paid = [...payments].filter(([, p]) => p.reference === call.params[0]);
        return { jsonrpc: '2.0', id: call.id, result: paid.map(([signature]) => ({ signature, err: null })) };
      }
      case 'getTransaction': {
        const { reference, memo, amount } = payments.get(call.params[0]);
        const tx = structuredClone(tip_tx);
        tx.result.transaction.message.accountKeys[0].pubkey = reference;
        tx.result.transaction.message.instructions.find((ix) => ix.program === 'spl-memo').parsed = memo;
        const pre = tx.result.meta.preTokenBalances.find((b) => b.owner === tip_wallet).uiTokenAmount.amount;
        tx.result.meta.postTokenBalances.find((b) => b.owner === tip_wallet).uiTokenAmount.amount = String(BigInt(pre) + BigInt(amount));
        return tx;
      }
      default:
        return { jsonrpc: '2.0', id: call.id, error: { code: -32601, message: 'not mocked' } };
    }
  });
  return { calls, pay: (reference, { signature, memo, amount }) => payments.set(signature, { reference, memo, amount }) };
}

const set_tip_wallet = async (page, wallet) => {
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Settings').click()`);
  await page.wait_for(`!!document.getElementById('tip_wallet')`);
  await page.eval(`(() => { const i = document.getElementById('tip_wallet'); i.value = ${JSON.stringify(wallet)}; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
};

test('tip QR: a wallet in Settings shows a Solana Pay code under the player that decodes to the link, fits TV and desktop; a tip on-chain shows under it', () => with_page({ width: 1920, height: 1080 }, async (page) => {
  const wallet = tip_wallet;
  const rpc = await mock_devnet(page);
  assert.equal(await page.eval(`document.querySelectorAll('.tip-strip').length`), 0, 'off until a wallet is set');
  await page.eval(`[...document.querySelectorAll('.shortcut-help button')].find((b) => b.textContent === 'Close').click()`);
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Settings').click()`);
  await page.wait_for(`!!document.getElementById('tip_wallet')`);
  await page.eval(`(() => { const i = document.getElementById('tip_wallet'); i.value = 'not a wallet'; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await sleep(200);
  assert.equal(await page.eval(`document.querySelectorAll('.tip-strip').length`), 0, 'an invalid address shows no code');
  await page.eval(`(() => { const i = document.getElementById('tip_wallet'); i.value = ' ${wallet} '; i.dispatchEvent(new Event('input', { bubbles: true })); })()`);
  await page.wait_for(`!!document.querySelector('.tip-qr')`);
  const href = await page.eval(`document.querySelector('.tip-qr-link').getAttribute('href')`);
  assert.match(href, new RegExp(`^solana:${wallet}\\?spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=[1-9A-HJ-NP-Za-km-z]{43,44}&label=[^&]+&message=Tip%20%23\\d+&memo=ktv%3A\\d+$`), 'devnet USDC by default');
  // The code itself decodes to the same link (BarcodeDetector exists on macOS / Android / ChromeOS Chrome, not Linux)
  const decoded = await page.eval(`(async () => {
    if (!('BarcodeDetector' in window)) return null;
    const svg = new XMLSerializer().serializeToString(document.querySelector('.tip-qr'));
    const img = new Image();
    img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg);
    await img.decode();
    const canvas = document.createElement('canvas');
    canvas.width = canvas.height = 600;
    canvas.getContext('2d').drawImage(img, 0, 0, 600, 600);
    const [code] = await new BarcodeDetector({ formats: ['qr_code'] }).detect(canvas);
    return code?.rawValue ?? 'not found';
  })()`);
  if (decoded !== null) assert.equal(decoded, href, 'QR decodes to the Solana Pay link');
  // S2: a guest pays; the booth finds it on-chain by the song's reference and shows it
  assert.equal(await page.eval(`document.querySelector('.tip-received').textContent`), '', 'no tip yet');
  const song_reference = new URL(href.replace('solana:', 'https://x/')).searchParams.get('reference');
  rpc.pay(song_reference, { signature: 'song-tip-1', memo: 'ktv:00001', amount: 1000 });
  await page.wait_for(`document.querySelector('.tip-received').textContent === '✓ Tip received: 0.001 USDC'`, 20000);
  assert.ok(rpc.calls.includes('getTransaction'));
  assert.match(await page.eval(`document.querySelector('.tip-toast').textContent`), /Garland for the singer! \+0\.001 USDC/);
  assert.equal(await page.eval(`document.querySelector('.tip-received').getAttribute('role')`), 'status');
  // A new song gets a new reference (each tip is matched to the song it was for)
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`);
  await page.wait_for(`document.querySelector('.tip-qr-link')?.getAttribute('href') !== ${JSON.stringify(href)}`);
  assert.equal(await page.eval(`document.querySelector('.tip-received').textContent`), '', 'tips belong to the song they were for');
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'TV').click()`);
  await page.wait_for(`document.querySelector('.ktv-app-wrapper').classList.contains('tv-mode')`);
  assert.equal(await page.eval(`document.documentElement.scrollHeight <= innerHeight`), true, 'TV mode still fits 1080p');
  assert.equal(await page.eval(`[...document.querySelectorAll('.player-main-controls-row button, .tip-qr')].filter((b) => b.getBoundingClientRect().bottom > innerHeight).length`), 0, 'controls and code on screen');
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'TV').click()`);
  await page.send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
  await sleep(300);
  assert.equal(await page.eval(`document.documentElement.scrollHeight <= innerHeight`), true, 'desktop 1280x800 still fits');
}));

test('tip request: the room QR opens the phone page; a paid request plays next as ★ TIP with a garland, once', () => with_page({ width: 1920, height: 1080 }, async (page) => {
  const rpc = await mock_devnet(page);
  await page.eval(`[...document.querySelectorAll('.shortcut-help button')].find((b) => b.textContent === 'Close').click()`);
  await set_tip_wallet(page, tip_wallet);
  await page.wait_for(`!!document.querySelector('.tip-request-qr')`);
  const request_page = await page.eval(`document.querySelector('.tip-request').href`);
  assert.ok(request_page.startsWith(`${await page.eval('location.origin')}/request#to=${tip_wallet}&c=devnet&ref=`), request_page);
  const room_reference = new URLSearchParams(new URL(request_page).hash.slice(1)).get('ref');
  assert.equal(await page.eval(`document.documentElement.scrollHeight <= innerHeight`), true, 'both QRs fit 1080p');

  // A guest pays 1 USDC with the request memo for a catalog song
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Songbook').click()`);
  await page.wait_for(`document.querySelectorAll('.song-code-tag').length > 0`);
  const code = (await page.eval(`[...document.querySelectorAll('.song-code-tag')].at(-1).textContent`)).replace('#', '');
  rpc.pay(room_reference, { signature: 'request-1', memo: `ktv:req:${code}`, amount: 1_000_000 });
  await page.wait_for(`document.querySelector('.tip-toast')?.textContent.includes('★ TIP 1.00 USDC')`, 20000);
  assert.match(await page.eval(`document.querySelector('.tip-toast').textContent`), /plays next/);
  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent.startsWith('Queue')).click()`);
  await page.wait_for(`!!document.querySelector('.queue-item-card .tip-badge')`);
  assert.equal(await page.eval(`document.querySelector('.queue-item-card .item-code').textContent`), `#${code}★ TIP`, 'first in the queue');
  // The next polls see the same signature again: it is not queued twice
  await page.click('.tip-toast .toast-close-btn');
  await sleep(6000);
  assert.equal(await page.eval(`document.querySelectorAll('.queue-item-card .tip-badge').length`), 1, 'acted on once');
  assert.equal(await page.eval(`!!document.querySelector('.tip-toast')`), false, 'no second toast');

  // The phone page builds the Solana Pay link for that room
  await page.send('Page.navigate', { url: request_page });
  await page.wait_for(`document.getElementById('form')?.hidden === false`);
  assert.equal(await page.eval(`document.getElementById('pay').getAttribute('aria-disabled')`), 'true', 'no code yet');
  await page.eval(`(() => { const i = document.getElementById('code'); i.value = '${code}x'; i.dispatchEvent(new Event('input')); })()`);
  await page.eval(`[...document.querySelectorAll('.amounts button')].find((b) => b.textContent === '2').click()`);
  assert.equal(await page.eval(`document.getElementById('pay').getAttribute('href')`),
    `solana:${tip_wallet}?amount=2&spl-token=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU&reference=${room_reference}`
    + `&label=${encodeURIComponent('VIP ROOM 07')}&message=Request%20%23${code}&memo=ktv%3Areq%3A${code}`);
  assert.equal(await page.eval(`document.getElementById('test_badge').hidden`), false, 'devnet is marked');
  // Or find the song by name: the index loads on first search, with the booth's codes
  await page.eval(`(() => { const i = document.getElementById('song_search'); i.value = 'รักไมไหวแลว'; i.dispatchEvent(new Event('input')); })()`);
  await page.wait_for(`document.querySelectorAll('#results button').length > 0`);
  assert.match(await page.eval(`document.querySelector('#results button').textContent`), /^10004รักไม่ไหวแล้วโว้ย/);
  await page.eval(`document.querySelector('#results button').click()`);
  assert.equal(await page.eval(`document.getElementById('code').value`), '10004');
  assert.match(await page.eval(`document.getElementById('pay').getAttribute('href')`), /&memo=ktv%3Areq%3A10004$/);
  assert.equal(await page.eval(`document.querySelectorAll('#results li').length`), 0, 'results close on pick');
}));

test('MC voice + mic: while the MC talks the mic scores nothing (its voice is not singing), then listens again', () => with_page({ fake_mic: true }, async (page) => {
  await page.wait_for(`!!localStorage.getItem('ktv.settings.v1')`);
  await page.eval(`(() => { const s = JSON.parse(localStorage.getItem('ktv.settings.v1')); s.mc_voice = 'english'; localStorage.setItem('ktv.settings.v1', JSON.stringify(s)); })()`);
  await page.reload();
  // The MC "talks" while window.__talking is set (headless Chrome has no voices)
  await page.eval(`(() => { window.__talking = true; Object.defineProperty(speechSynthesis, 'speaking', { get: () => window.__talking }); })()`);
  await page.eval(`[...document.querySelectorAll('.pitch-meter button')].find((b) => b.textContent.startsWith('Mic')).click()`);
  await page.wait_for(`!!window.__mic_gain`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent === 'Room check…'`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent !== 'Room check…'`, 6000);
  await page.eval(`window.__mic_gain.gain.value = 0.3`);
  await sleep(800);
  assert.equal(await page.eval(`document.querySelector('.pitch-note').textContent`), '—', 'not read while the MC talks');
  await page.eval(`window.__talking = false`);
  await page.wait_for(`document.querySelector('.pitch-note')?.textContent === 'A3'`, 3000);
}));

test('MC voice: off by default; when on, each new song is announced once in the chosen language', () => with_page({}, async (page) => {
  // Record instead of speaking; headless Chrome has no voices, so the test supplies them (network-only at first)
  await page.eval(`(() => {
    const voice = (lang, localService) => Object.defineProperties(Object.create(SpeechSynthesisVoice.prototype), {
      lang: { value: lang }, localService: { value: localService }, name: { value: lang }, voiceURI: { value: lang }, default: { value: false } });
    window.__voices = [voice('en-US', false)];
    window.__local_voices = () => { window.__voices = [voice('en-US', false), voice('th-TH', true), voice('en-US', true)]; };
    speechSynthesis.getVoices = () => window.__voices;
    // The real utterance only takes a real voice; a plain one takes the stub
    window.SpeechSynthesisUtterance = class { constructor(text) { this.text = text; } };
    window.__spoken = [];
    speechSynthesis.speak = (u) => window.__spoken.push({ lang: u.lang, text: u.text, local: u.voice?.localService });
  })()`);
  await page.eval(`[...document.querySelectorAll('.shortcut-help button')].find((b) => b.textContent === 'Close').click()`);
  const next_song = `[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Next Song').click()`;
  await page.eval(next_song);
  await sleep(1000);
  assert.deepEqual(await page.eval('window.__spoken'), [], 'silent until chosen');

  await page.eval(`[...document.querySelectorAll('.nav-btn')].find((b) => b.textContent === 'Settings').click()`);
  await page.wait_for(`!!document.getElementById('mc_voice')`);
  await page.click('#mc_voice');
  await page.wait_for(`document.getElementById('mc_voice').textContent === 'ไทย'`);
  // No Thai voice on the device: silent, never a network voice
  await page.eval(next_song);
  await sleep(1000);
  assert.deepEqual(await page.eval('window.__spoken'), [], 'no local voice, nothing said');
  await page.eval('window.__local_voices()');
  await page.eval(next_song);
  await page.wait_for('window.__spoken.length === 1');
  const title = await page.eval(`document.querySelector('.now-title').textContent`);
  const [said] = await page.eval('window.__spoken');
  assert.equal(said.lang, 'th-TH');
  assert.equal(said.local, true, 'an on-device voice');
  assert.ok(said.text.includes(title), `"${said.text}" names the song on stage, "${title}"`);
  // A replay is the same song: not announced again
  await page.eval(`[...document.querySelectorAll('.player-main-controls-row button')].find((b) => b.textContent === 'Replay').click()`);
  await sleep(1000);
  assert.equal(await page.eval('window.__spoken.length'), 1, 'replay is not re-announced');

  await page.click('#mc_voice');
  await page.wait_for(`document.getElementById('mc_voice').textContent === 'English'`);
  await page.eval(next_song);
  await page.wait_for('window.__spoken.length === 2');
  assert.equal(await page.eval('window.__spoken[1].lang'), 'en-US');
}));
