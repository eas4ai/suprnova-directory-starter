import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { writeFile, rm, readFile } from 'node:fs/promises';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run with the built SSR worker');
assert.ok(process.env.CARGO_TARGET_DIR, 'Use the isolated acceptance workspace');
const children = [];
let browser;
let logs = '';
async function freePort() {
  const probe = createServer();
  await new Promise((resolve, reject) => { probe.once('error', reject); probe.listen(0, '127.0.0.1', resolve); });
  const port = probe.address().port;
  await new Promise((resolve, reject) => probe.close(error => error ? reject(error) : resolve()));
  return port;
}
function launch(command, args, env, cwd = process.cwd()) {
  const child = spawn(command, args, { env, cwd, stdio: ['ignore', 'pipe', 'pipe'] });
  children.push(child);
  child.on('error', error => { logs += error.stack; });
  for (const stream of [child.stdout, child.stderr]) stream.on('data', data => { logs = (logs + data).slice(-24000); });
  return child;
}
async function ready(child, url) {
  let lastError;
  for (let attempt = 0; attempt < 150; attempt++) {
    assert.equal(child.exitCode, null, logs);
    try { if ((await fetch(url, { signal: AbortSignal.timeout(500) })).ok) return; } catch (error) { lastError = error; }
    await delay(100);
  }
  throw new Error(`Server did not become ready: ${url}\n${logs}`, { cause: lastError });
}
async function stop(child) {
  if (!child.pid || child.exitCode !== null || child.signalCode !== null) return;
  child.kill('SIGTERM');
  await Promise.race([new Promise(resolve => child.once('exit', resolve)), delay(3000)]);
  if (child.exitCode === null && child.signalCode === null) {
    child.kill('SIGKILL');
    await new Promise(resolve => child.once('exit', resolve));
  }
}
function privateResponse(response, path) {
  const cache = response.headers.get('cache-control') ?? '';
  assert.match(cache, /(?:private|no-store)/i, `${path} must prohibit shared caching: ${cache}`);
  assert.doesNotMatch(cache, /(?:^|,)\s*public(?:,|$)/i, path);
}
const port = await freePort();
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin,
  SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true',
  RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
const binary = join(env.CARGO_TARGET_DIR, 'debug/directory');
const watchdog = setTimeout(() => {
  for (const child of children) child.kill('SIGKILL');
  void browser?.close();
  process.exitCode = 1;
}, 120000);
try {
  const migration = spawnSync(binary, ['migrate'], { env, encoding: 'utf8', timeout: 30000 });
  assert.equal(migration.status, 0, migration.stderr);
  // A stale hot file must never divert production traffic to a dev server.
  await writeFile('public/hot', 'http://127.0.0.1:1');
  const production = launch(binary, ['serve', '--no-migrate'], env);
  await ready(production, origin + '/_suprnova/health');
  const manifest = JSON.parse(await readFile('public/assets/.vite/manifest.json', 'utf8'));
  const cssPath = '/assets/' + manifest['src/app.css'].file;
  const response = await fetch(origin);
  assert.equal(response.status, 200);
  privateResponse(response, '/');
  const html = await response.text();
  assert.ok(html.includes(cssPath), 'Production HTML must link the explicit stylesheet');
  assert.doesNotMatch(html, /127\.0\.0\.1:1["/]|@vite\/client/);
  browser = await chromium.launch();
  const noJs = await browser.newPage({ javaScriptEnabled: false });
  await noJs.goto(origin);
  await expect(noJs.getByRole('heading', { name: 'A place for good discoveries.' })).toBeVisible();
  await expect(noJs.locator('link[rel="canonical"]')).toHaveAttribute('href', origin + '/');
  assert.ok(await noJs.title(), 'SSR must emit a title');
  assert.equal(await noJs.locator('meta[property="og:title"]').count(), 1);
  assert.equal(await noJs.locator('body').evaluate(el => getComputedStyle(el).backgroundColor), 'rgb(255, 255, 255)');
  for (const path of ['/login', '/register', '/reset-password?token=fixture&email=fixture@example.test']) {
    const tokenResponse = await fetch(origin + path);
    assert.equal(tokenResponse.status, 200, path);
    privateResponse(tokenResponse, path);
  }
  // Static asset requests must succeed; explicit public-cache behavior is
  // exercised separately through the application middleware in the Rust suite.
  const asset = await fetch(origin + cssPath);
  assert.equal(asset.status, 200);

  assert.ok((await asset.text()).length > 100);

  const page = await browser.newPage();
  await page.goto(origin + '/register');
  await page.getByLabel('Name', { exact: true }).fill('Upgrade cache fixture');
  await page.getByLabel('Email address', { exact: true }).fill('cache@example.test');
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByLabel('Confirm Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Register', exact: true }).click();
  await expect(page).toHaveURL(origin + '/dashboard');
  const initialPage = JSON.parse(await noJs.locator('script[type="application/json"][data-page="app"]').textContent());
  const version = initialPage.version;
  const cookie = (await page.context().cookies()).map(({ name, value }) => `${name}=${value}`).join('; ');
  for (const path of ['/', '/dashboard', '/verify-email']) {
    for (const headers of [{ cookie }, { cookie, 'X-Inertia': 'true', 'X-Inertia-Version': version, 'X-Requested-With': 'XMLHttpRequest' }]) {
      const personalized = await fetch(origin + path, { headers });
      assert.equal(personalized.status, 200, path);
      privateResponse(personalized, path);
      assert.ok((await personalized.text()).includes('cache@example.test'), `${path} must exercise signed-in props`);
    }
  }
  const signedInAsset = await fetch(origin + cssPath, { headers: { cookie } });
  assert.equal(signedInAsset.status, 200);

  await stop(production);

  const vitePort = await freePort();
  const viteOrigin = `http://127.0.0.1:${vitePort}`;
  const vite = launch(process.execPath, ['node_modules/vite/bin/vite.js', '--host', '127.0.0.1', '--port', String(vitePort)], env, join(process.cwd(), 'frontend'));
  await ready(vite, viteOrigin + '/@vite/client');
  // Vite can answer asset requests while its SSR module graph is warming up.
  // Wait for a real render, not merely a listening socket or a fixed delay.
  await expect.poll(async () => {
    const render = await fetch(viteOrigin + '/__inertia_ssr', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(initialPage), signal: AbortSignal.timeout(10000),
    });
    assert.equal(render.status, 200, await render.clone().text());
    return (await render.json())?.body?.includes('data-shell="public"') ?? false;
  }, { timeout: 30000 }).toBe(true);
  await writeFile('public/hot', viteOrigin);
  const development = launch(binary, ['serve', '--no-migrate'], { ...env, APP_ENV: 'development' });
  await ready(development, origin + '/_suprnova/health');
  const devResponse = await fetch(origin);
  assert.equal(devResponse.status, 200);
  const devHtml = await devResponse.text();
  assert.ok(devHtml.includes(viteOrigin + '/src/app.css'), 'Hot file must select the live CSS entry');
  assert.ok(devHtml.includes(viteOrigin + '/@vite/client'), 'Hot file must select the live Vite client');
  await noJs.goto(origin);
  await expect(noJs.getByRole('heading', { name: 'A place for good discoveries.' })).toBeVisible();
  assert.equal(await noJs.locator('body').evaluate(el => getComputedStyle(el).backgroundColor), 'rgb(255, 255, 255)');
  console.log('Production assets/SSR, development hot file, personalized cache protection and cookie-free public responses passed.');
} catch (error) {
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  for (const child of children.reverse()) await stop(child);
  await rm('public/hot', { force: true });
}
