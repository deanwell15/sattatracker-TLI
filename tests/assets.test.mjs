// Real Chromium rendering; Tauri IPC is mocked in assets-harness.ts.
// Uses an already installed browser. No browser or executable downloads.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, rm, readFile, readdir } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import './asset-integrity.mjs';
import { once } from 'node:events';
import { createServer } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

const browser = [process.env.CHROME_BIN,
  ...['ProgramFiles', 'ProgramFiles(x86)', 'LOCALAPPDATA'].map(key => process.env[key] && join(process.env[key], 'Google/Chrome/Application/chrome.exe')),
  '/usr/bin/chromium', '/usr/bin/chromium-browser', '/usr/bin/google-chrome',
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
].find(path => path && existsSync(path));
assert(browser, 'Install Chrome/Chromium or set CHROME_BIN; tests cannot be skipped.');

// Guard executable source/config against accidentally restoring upstream paths.
const paths = ['index.html'];
for (const root of ['src', 'src-tauri/src']) {
  paths.push(...(await readdir(root, { recursive: true })).map(path => join(root, path)));
}
for (const path of paths) {
  if (!existsSync(path) || !/\.(vue|ts|rs|html)$/.test(path)) continue;
  assert(!/sattatrapy\/sattatrackerUpdate|assets\.zip|public_assets|assets_version\.json|check_assets_update|download_assets|get_assets_path|fonts\.googleapis|fonts\.gstatic/.test(await readFile(path, 'utf8')), `Legacy asset dependency: ${path}`);
}
const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
assert.equal(config.app.security.assetProtocol.enable, false);
assert.deepEqual(config.app.security.assetProtocol.scope, []);
assert.deepEqual(config.bundle.resources, { 'resources/icons/*.png': 'icons/' });

const server = await createServer({
  configFile: false,
  plugins: [vue(), tailwindcss(), {
    name: 'asset-fault-injection',
    configureServer(vite) {
      vite.middlewares.use(async (req, res, next) => {
        try {
          const path = new URL(req.url || '/', 'http://127.0.0.1').pathname;
          const isImage = req.headers['sec-fetch-dest'] === 'image';
          if (isImage && (path.startsWith('/src/assets/') || path.startsWith('/__icons/'))) {
            const mode = /asset-test=(missing|corrupt)/.exec(req.headers.cookie || '')?.[1];
            if (mode) {
              res.statusCode = mode === 'missing' ? 404 : 200;
              res.setHeader('Content-Type', 'image/png');
              res.setHeader('Cache-Control', 'no-store');
              res.end(mode === 'missing' ? '' : 'not an image');
              return;
            }
          }
          if (path.startsWith('/__icons/')) {
            const name = path.slice('/__icons/'.length);
            if (!/^(?:[0-9]{1,12}|fe)\.png$/.test(name)) { res.statusCode=400; res.end(); return; }
            const bytes = await readFile(`src-tauri/resources/icons/${name}`);
            res.setHeader('Content-Type', bytes.toString('ascii',0,4)==='RIFF' ? 'image/webp' : 'image/png');
            res.setHeader('Cache-Control', 'no-store');
            res.end(bytes);
            return;
          }
          next();
        } catch (error) { next(error); }
      });
    },
  }],
  server: { host: '127.0.0.1', port: 0, strictPort: false, hmr: false },
  clearScreen: false,
});
const temp = await mkdtemp(join(tmpdir(), 'deanfinite-assets-'));
let chrome;
let socket;
let closeBrowser;
try {
  await server.listen();
  const origin = `http://127.0.0.1:${server.httpServer.address().port}`;
  chrome = spawn(browser, ['--headless=new', '--disable-gpu', '--disable-background-networking', '--no-first-run', '--no-default-browser-check',
    ...(process.platform === 'linux' ? ['--no-sandbox'] : []),
    '--remote-debugging-port=0', `--user-data-dir=${temp}`, 'about:blank'], { stdio: ['ignore', 'ignore', 'pipe'] });
  const endpoint = await new Promise((resolve, reject) => {
    let output = '';
    const timer = setTimeout(() => reject(new Error(`Browser startup timeout: ${output.slice(-1000)}`)), 20000);
    chrome.once('error', error => { clearTimeout(timer); reject(error); });
    chrome.once('exit', code => { clearTimeout(timer); reject(new Error(`Browser exited ${code}: ${output.slice(-1000)}`)); });
    chrome.stderr.on('data', data => {
      output += data;
      const match = /DevTools listening on (ws:\/\/\S+)/.exec(output);
      if (match) { clearTimeout(timer); resolve(match[1]); }
    });
  });
  socket = new WebSocket(endpoint);
  await once(socket, 'open');
  let id = 0;
  const pending = new Map();
  const external = [];
  socket.addEventListener('message', event => {
    const msg = JSON.parse(event.data);
    if (msg.method === 'Fetch.requestPaused') {
      const allowed = msg.params.request.url.startsWith(`${origin}/`);
      if (!allowed) external.push(msg.params.request.url);
      void cdp(allowed ? 'Fetch.continueRequest' : 'Fetch.failRequest', {
        requestId: msg.params.requestId,
        ...(!allowed ? { errorReason: 'InternetDisconnected' } : {}),
      }, msg.sessionId).catch(error => external.push(String(error)));
      return;
    }
    if (!msg.id) return;
    const entry = pending.get(msg.id);
    if (!entry) return;
    clearTimeout(entry.timer);
    pending.delete(msg.id);
    msg.error ? entry.reject(new Error(JSON.stringify(msg.error))) : entry.resolve(msg.result);
  });
  function cdp(method, params = {}, sessionId) {
    return new Promise((resolve, reject) => {
      const call = ++id;
      const timer = setTimeout(() => { pending.delete(call); reject(new Error(`CDP timeout: ${method}`)); }, 10000);
      pending.set(call, { resolve, reject, timer });
      socket.send(JSON.stringify({ id: call, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  }
  closeBrowser = () => cdp('Browser.close');
  let cleanContext;
  for (const scenario of ['clean', 'restart', 'offline', 'missing', 'corrupt', 'widget']) {
    // Restart reuses the clean profile's storage; others get empty browser contexts.
    const context = scenario === 'restart' ? undefined : (await cdp('Target.createBrowserContext')).browserContextId;
    const target = await cdp('Target.createTarget', { url: 'about:blank', browserContextId: scenario === 'restart' ? cleanContext : context });
    if (scenario === 'clean') cleanContext = context;
    const { sessionId } = await cdp('Target.attachToTarget', { targetId: target.targetId, flatten: true });
    await cdp('Page.enable', {}, sessionId);
    await cdp('Network.enable', {}, sessionId);
    await cdp('Network.setCacheDisabled', { cacheDisabled: true }, sessionId);
    // Block every non-loopback HTTP(S) request, including any restored Google/upstream links.
    await cdp('Fetch.enable', { patterns: [{ urlPattern: 'http://*' }, { urlPattern: 'https://*' }] }, sessionId);
    await cdp('Page.navigate', { url: `${origin}/tests/assets-harness.html?scenario=${scenario}${scenario === 'widget' ? '&widgetOnly=lootFeedSession' : ''}` }, sessionId);
    let result = '';
    for (let attempt = 0; attempt < 900; attempt++) {
      const evaluated = await cdp('Runtime.evaluate', { expression: 'document.querySelector("#result")?.textContent || ""', returnByValue: true }, sessionId);
      result = evaluated.result.value || '';
      if (/^(PASS|FAIL)/.test(result)) break;
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    assert.equal(result, `PASS ${scenario}`, result || `Timeout: ${scenario}`);
    assert.deepEqual(external, [], 'Unexpected external requests');
    console.log(result);
    await cdp('Target.closeTarget', { targetId: target.targetId });
  }
  console.log('PASS source/config guards; 6 browser scenarios (Tauri mocked)');
} finally {
  if (chrome && chrome.exitCode === null) {
    const exited = once(chrome, 'exit');
    if (closeBrowser && socket?.readyState === WebSocket.OPEN) {
      await closeBrowser().catch(() => {});
    } else {
      chrome.kill();
    }
    await Promise.race([exited, new Promise(resolve => setTimeout(resolve, 3000))]);
    if (chrome.exitCode === null) chrome.kill();
  }
  socket?.close();
  await server.close();
  await rm(temp, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
}
