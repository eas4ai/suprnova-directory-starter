import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { readFileSync, existsSync } from 'node:fs';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const guide = readFileSync('README.md', 'utf8');
function block(name) {
  const expression = new RegExp(`<!-- foundation:${name} -->\\s*\x60\x60\x60sh\\n([\\s\\S]*?)\x60\x60\x60`, 'g');
  const matches = [...guide.matchAll(expression)];
  assert.equal(matches.length, 1, `README must have one ${name} command block`);
  return matches[0][1];
}

const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME',
  'TMPDIR', 'CARGO_TARGET_DIR', 'CARGO_BUILD_JOBS', 'CARGO_PROFILE_DEV_DEBUG']
  .filter(key => process.env[key]).map(key => [key, process.env[key]]));
const children = [];
let browser;
let logs = '';

async function freePort() {
  const server = createServer();
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return port;
}

function start(name) {
  const command = block(name);
  console.log(`[setup:${name}]\n${command}`);
  const child = spawn('sh', ['-eu', '-c', command], { env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  children.push(child);
  child.on('error', error => { logs += `${name}: ${error.message}\n`; });
  for (const stream of [child.stdout, child.stderr]) stream.on('data', data => { logs = (logs + data).slice(-16000); });
  return child;
}

function signalGroup(child, signal) {
  if (!child.pid) return;
  try { process.kill(-child.pid, signal); } catch (error) { if (error.code !== 'ESRCH') throw error; }
}

async function ready(url, child) {
  for (let attempt = 0; attempt < 150; attempt++) {
    assert.equal(child.exitCode, null, `README server command exited: ${logs}`);
    try { if ((await fetch(url, { signal: AbortSignal.timeout(500) })).ok) return; } catch { /* Poll until the bounded startup deadline. */ }
    await delay(100);
  }
  throw new Error(`README server did not become ready at ${url}: ${logs}`);
}

try {
  assert.ok(!existsSync('.env') && !existsSync('database.db'), 'Setup must start without operator state');
  const command = block('install');
  console.log(`[setup:install]\n${command}`);
  const installed = spawnSync('sh', ['-eu', '-c', command], { env, stdio: 'inherit', timeout: 30 * 60 * 1000 });
  if (installed.error) throw installed.error;
  assert.equal(installed.status, 0, 'README install commands failed');
  assert.ok(existsSync('database.db'), 'README did not initialize its example database');
  assert.match(readFileSync('.env', 'utf8'), /^APP_KEY=[A-Za-z0-9_-]{43}$/m, 'README did not generate a local key');
  const schema = spawnSync('bun', ['scripts/check-schema.mjs', 'database.db'], { env, encoding: 'utf8', timeout: 10000 });
  assert.equal(schema.status, 0, `README database schema is incomplete: ${schema.stderr}`);

  env.SERVER_PORT = String(await freePort());
  env.VITE_PORT = String(await freePort());
  env.APP_URL = `http://localhost:${env.SERVER_PORT}`;
  const vite = start('vite');
  await ready(`http://localhost:${env.VITE_PORT}/@vite/client`, vite);
  const app = start('serve');
  await ready(`${env.APP_URL}/_suprnova/health`, app);

  const { chromium, expect } = await import('@playwright/test');
  browser = await chromium.launch();
  const page = await browser.newPage();
  page.setDefaultTimeout(10000);
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(env.APP_URL);
  await expect(page.locator('[data-shell="public"]')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'A place for good discoveries.' })).toBeVisible();
  await page.getByRole('link', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(`${env.APP_URL}/login`);
  await page.getByLabel('Email address', { exact: true }).fill('not-registered@example.test');
  await page.getByLabel('Password', { exact: true }).fill('not-a-real-password');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('alert')).toBeVisible();
  await expect(page).toHaveURL(`${env.APP_URL}/login`);
  expect(errors).toEqual([]);
  console.log('README install, example migration, Vite, application and browser form checks passed.');
} catch (error) {
  console.error(logs);
  throw error;
} finally {
  await browser?.close();
  for (const child of children) signalGroup(child, 'SIGTERM');
  await Promise.all(children.map(async child => {
    if (child.exitCode !== null || child.signalCode !== null) return;
    await Promise.race([new Promise(resolve => child.once('exit', resolve)), delay(3000)]);
  }));
  // Kill any descendants even if their shell has already exited.
  for (const child of children) signalGroup(child, 'SIGKILL');
}
