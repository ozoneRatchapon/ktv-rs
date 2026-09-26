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
