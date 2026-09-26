// Minimal Chrome DevTools Protocol driver for the e2e tests: no npm dependencies, Node 24 built-ins only.
// KTV_URL = app under test (default: local `wrangler dev`), CHROME_BIN = browser binary.
import { spawn } from 'node:child_process';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

export const app_url = process.env.KTV_URL ?? 'http://localhost:8788/';
const chrome_bin = process.env.CHROME_BIN ??
  (process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : 'google-chrome');

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// A sawtooth "voice" instead of a microphone (headless Chrome has none). Starts silent so the
// app's room check measures a quiet room; set `window.__mic_gain.gain.value` to sing.
const FAKE_MIC = `navigator.mediaDevices.getUserMedia = async () => {
  const ctx = new AudioContext();
  const osc = new OscillatorNode(ctx, { type: 'sawtooth', frequency: 220 });
  const gain = new GainNode(ctx, { gain: 0 });
  const out = ctx.createMediaStreamDestination();
  osc.connect(gain).connect(out);
  osc.start();
  window.__mic = osc;
  window.__mic_gain = gain;
  return out.stream;
};`;

/** Launch headless Chrome; returns { new_page, close }. Each page gets its own browser context (fresh storage). */
export async function launch() {
  const profile = mkdtempSync(join(tmpdir(), 'ktv-e2e-'));
  const args = ['--headless=new', '--remote-debugging-port=0', `--user-data-dir=${profile}`, '--no-first-run',
    '--no-default-browser-check', '--mute-audio', '--autoplay-policy=no-user-gesture-required', 'about:blank'];
  if (process.env.CI) args.push('--no-sandbox');
  const proc = spawn(chrome_bin, args, { stdio: ['ignore', 'ignore', 'pipe'] });
  const ws_url = await new Promise((resolve, reject) => {
    let buf = '';
    proc.stderr.on('data', (d) => {
      buf += d;
      const m = buf.match(/DevTools listening on (ws:\/\/\S+)/);
      if (m) resolve(m[1]);
    });
    proc.on('exit', (code) => reject(new Error(`chrome exited ${code}: ${buf}`)));
  });
  const browser = await connect(ws_url);

  /** `fake_mic`: getUserMedia returns an oscillator: `window.__mic.frequency.value = hz`, `window.__mic_gain.gain.value = level`. */
  async function new_page({ width = 1280, height = 900, fake_mic = false } = {}) {
    const { browserContextId } = await browser.send('Target.createBrowserContext');
    const { targetId } = await browser.send('Target.createTarget', { url: 'about:blank', browserContextId });
    const { sessionId } = await browser.send('Target.attachToTarget', { targetId, flatten: true });
    const page = session_client(browser, sessionId);
    page.errors = [];
    browser.listeners.add((msg) => {
      if (msg.sessionId !== sessionId) return;
      if (msg.method === 'Runtime.exceptionThrown') page.errors.push(msg.params.exceptionDetails.exception?.description ?? msg.params.exceptionDetails.text);
      if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') page.errors.push(msg.params.args.map((a) => a.value ?? a.description).join(' '));
    });
    await page.send('Runtime.enable');
    await page.send('Page.enable');
    // CSP violations never reach the console reliably; record them in the page
    await page.send('Page.addScriptToEvaluateOnNewDocument', {
      source: `window.__csp = []; document.addEventListener('securitypolicyviolation', (e) => window.__csp.push(e.violatedDirective + ' ' + e.blockedURI));`,
    });
    if (fake_mic) {
      await page.send('Page.addScriptToEvaluateOnNewDocument', { source: FAKE_MIC });
    }
    await page.send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
    page.close = () => browser.send('Target.disposeBrowserContext', { browserContextId });
    return page;
  }

  async function close() {
    browser.ws.close();
    proc.kill();
    await new Promise((r) => proc.once('exit', r));
    rmSync(profile, { recursive: true, force: true });
  }
  return { new_page, close };
}

async function connect(ws_url) {
  const ws = new WebSocket(ws_url);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  const pending = new Map();
  const listeners = new Set();
  let next_id = 0;
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result);
    }
    for (const l of listeners) l(msg);
  };
  const send = (method, params = {}, sessionId) => new Promise((resolve, reject) => {
    const id = ++next_id;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params, ...(sessionId && { sessionId }) }));
  });
  return { ws, send, listeners };
}

function session_client(browser, sessionId) {
  const send = (method, params) => browser.send(method, params, sessionId);
  const page = {
    send,
    /** Evaluate an expression (awaits promises) and return its JSON value; throws on page exceptions. */
    async eval(expression) {
      const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
      if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
      return r.result.value;
    },
    /** Poll until the expression is truthy (the wasm app renders asynchronously). */
    async wait_for(expression, timeout_ms = 15000) {
      const until = Date.now() + timeout_ms;
      while (Date.now() < until) {
        try { if (await page.eval(expression)) return; } catch { /* page still loading */ }
        await sleep(100);
      }
      throw new Error(`timed out waiting for: ${expression}`);
    },
    async goto(url = app_url) {
      await send('Page.navigate', { url });
      await page.wait_for(`!!document.querySelector('.ktv-app-wrapper')`);
    },
    async reload() {
      await send('Page.reload');
      await sleep(200);
      await page.wait_for(`!!document.querySelector('.ktv-app-wrapper')`);
    },
    async key(key) {
      await send('Input.dispatchKeyEvent', { type: 'keyDown', key, text: key.length === 1 ? key : undefined });
      await send('Input.dispatchKeyEvent', { type: 'keyUp', key });
    },
    async click(selector) {
      await page.eval(`document.querySelector(${JSON.stringify(selector)}).click()`);
    },
  };
  return page;
}
