import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import http from 'node:http';
import { spawn } from 'node:child_process';
import sharp from 'sharp';

// "Continue Watching" against a loopback stand-in for metahub and the Stremio API. Never
// contacts the real services.
const root = path.resolve(import.meta.dirname, '..');
const out = path.join(root, '.build');
await fs.mkdir(out, { recursive: true });
const temporary = await fs.mkdtemp(path.join(out, 'for-you-smoke-'));
const preferences = path.join(temporary, 'preferences.json');
const authFile = path.join(temporary, 'preferences.stremio-auth.json');
const binary = path.resolve(root, process.env.CARGO_TARGET_DIR ?? 'target', 'debug/luma');

const poster = (hue) =>
  sharp({
    create: { width: 300, height: 450, channels: 3, background: { r: hue, g: 60, b: 255 - hue } },
  })
    .jpeg()
    .toBuffer();
const saved = (id, type, name, offset, episode, last) => ({
  _id: id,
  type,
  name,
  removed: false,
  temp: false,
  state: {
    timeOffset: offset,
    duration: 3_600_000,
    season: episode[0],
    episode: episode[1],
    lastWatched: last,
  },
});
let library = {
  result: [
    saved('tt1000002', 'movie', 'Shared Title', 900_000, [0, 0], '2026-11-07T10:00:00.000Z'),
    saved('tt1000009', 'series', 'Resumed Series', 1_800_000, [2, 3], '2026-11-08T10:00:00.000Z'),
  ],
};
const requests = [];
const server = http.createServer(async (request, response) => {
  const url = new URL(request.url, 'http://127.0.0.1');
  let body = '';
  for await (const chunk of request) body += chunk;
  requests.push({ method: request.method, path: url.pathname, body });
  response.on('error', () => {});
  const json = (value) => {
    response.writeHead(200, { 'Content-Type': 'application/json' });
    response.end(JSON.stringify(value));
  };
  if (url.pathname === '/api/datastoreGet' && request.method === 'POST') return json(library);
  const image = url.pathname.match(/^\/poster\/small\/tt(\d+)\/img$/);
  if (image) {
    response.writeHead(200, { 'Content-Type': 'image/jpeg' });
    return response.end(await poster(Number(image[1]) % 200));
  }
  response.writeHead(404);
  response.end();
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;

async function run(command, args) {
  await new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: root, stdio: 'inherit' });
    const timeout = setTimeout(() => {
      child.kill('SIGKILL');
      reject(new Error('For You smoke child timed out'));
    }, 60000);
    child.on('error', (error) => {
      clearTimeout(timeout);
      reject(error);
    });
    child.on('exit', (code) => {
      clearTimeout(timeout);
      if (code === 0) resolve();
      else reject(new Error(`${command} exited ${code}`));
    });
  });
}
async function scenario(name, script = 'home', frames = 240) {
  const dump = path.join(temporary, `${name}.json`);
  const screenshot = path.join(out, `for-you-${name}.png`);
  await run(binary, [
    '--preferences',
    preferences,
    '--for-you-test-server',
    origin,
    '--script',
    script,
    '--frames',
    String(frames),
    '--time',
    '2026-11-08T21:00:00-03:00',
    '--screenshot',
    screenshot,
    '--dump-state',
    dump,
  ]);
  const state = JSON.parse(await fs.readFile(dump, 'utf8'));
  assert.equal(state.error, null);
  assert.equal((await sharp(screenshot).stats()).channels[3].min, 255);
  return state;
}
const libraryCalls = () => requests.filter((r) => r.path === '/api/datastoreGet');

try {
  if (process.env.LUMA_SMOKE_SKIP_BUILD !== '1')
    await run('cargo', ['build', '--locked', '--package', 'luma']);
  await fs.writeFile(preferences, JSON.stringify({ language: 'en' }));

  const signedOut = await scenario('signed-out');
  assert.equal(libraryCalls().length, 0, 'No key, no library request');
  assert.ok(!signedOut.ui_text.includes('Continue Watching'), 'no key, no row');
  assert.ok(!signedOut.ui_text.includes('For You'), 'the catalog row is gone');

  await fs.writeFile(authFile, JSON.stringify({ authKey: 'smoke-key' }));
  const resumed = await scenario('continue-watching', 'home,down');
  assert.equal(libraryCalls().length, 1);
  assert.equal(
    requests.filter((r) => r.path.startsWith('/catalog/')).length,
    0,
    'the catalog is never fetched',
  );
  const call = JSON.parse(libraryCalls()[0].body);
  assert.deepEqual([call.authKey, call.collection, call.all], ['smoke-key', 'libraryItem', true]);
  assert.ok(resumed.ui_text.includes('Continue Watching'), resumed.ui_text);
  assert.deepEqual(resumed.focus, { ForYou: 'tt1000009' }, 'the latest title in progress first');
  assert.ok(resumed.ui_text.includes('Resumed Series  ·  S2 E3 · 30m left'), resumed.ui_text);
  const second = await scenario('continue-watching-second', 'home,down,right');
  assert.deepEqual(second.focus, { ForYou: 'tt1000002' });
  assert.ok(second.ui_text.includes('Shared Title  ·  45m left'), second.ui_text);

  library = { error: { message: 'Session not found', code: 1 } };
  const rejected = await scenario('rejected-key');
  assert.ok(!rejected.ui_text.includes('Continue Watching'), 'a rejected key shows no row');
  assert.equal(JSON.stringify(rejected).includes('smoke-key'), false, 'the key is never dumped');

  console.log(
    'Continue Watching smoke checks passed: no row without a key, titles in progress with episode and time left, and a rejected key.',
  );
} finally {
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
  await fs.rm(temporary, { recursive: true, force: true });
}
