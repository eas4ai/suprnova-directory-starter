import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run SEO journeys with the real SSR renderer');
assert.ok(process.env.CARGO_TARGET_DIR, 'CARGO_TARGET_DIR is required');
const probe = createServer();
await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin, SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
let server, browser;
let logs = '';
const errors = [];

async function start() {
  server = spawn(join(env.CARGO_TARGET_DIR, 'debug/directory'), ['serve', '--no-migrate'], { env, stdio: ['ignore', 'pipe', 'pipe'] });
  for (const stream of [server.stdout, server.stderr]) stream.on('data', data => { logs = (logs + data).slice(-20000); });
  let lastError;
  for (let attempt = 0; attempt < 150; attempt++) {
    if (server.exitCode !== null) throw new Error(`Application exited: ${logs}`);
    try { if ((await fetch(`${origin}/_suprnova/health`, { signal: AbortSignal.timeout(500) })).ok) return; }
    catch (error) { lastError = error; }
    await delay(100);
  }
  throw new Error(`Application did not start: ${logs}`, { cause: lastError });
}

async function stop() {
  if (!server || server.exitCode !== null || server.signalCode !== null) return;
  server.kill('SIGTERM');
  await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(6000)]);
  if (server.exitCode === null && server.signalCode === null) server.kill('SIGKILL');
}

async function login(page, name, path = '/admin') {
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (/hydration/i.test(message.text())) errors.push(message.text()); });
  await page.goto(origin + '/login');
  await page.getByLabel('Email address', { exact: true }).fill(`seo-${name}@example.test`);
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(origin + '/dashboard');
  await page.goto(origin + path);
}

async function screenshot(page, name) {
  if (!env.SEO_ARTIFACT_DIR) return;
  await mkdir(env.SEO_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.SEO_ARTIFACT_DIR, name), fullPage: true, animations: 'disabled' });
}


async function head(page, path) {
  await expect(page.locator('head link[rel="canonical"]')).toHaveCount(1);
  await expect(page.locator('head link[rel="canonical"]')).toHaveAttribute('href', origin + path);
  for (const selector of ['meta[name="description"]', 'meta[name="robots"]', 'meta[property="og:title"]', 'meta[property="og:description"]', 'meta[name="twitter:title"]', 'meta[name="twitter:description"]']) await expect(page.locator('head ' + selector)).toHaveCount(1);
  await expect(page.locator('head meta[name="robots"]')).toHaveAttribute('content', /index, follow/);
}
const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  await login(page, 'admin', '/admin/seo');
  await expect(page.getByLabel('Title format', { exact: true })).toHaveValue('{title} · {site}');
  await page.getByLabel('Default description', { exact: true }).fill('Browser saved site description.');
  await page.getByRole('button', { name: 'Save SEO settings', exact: true }).click();
  await expect(page.getByRole('status')).toHaveText('SEO settings saved.');
  await page.reload();
  await expect(page.getByLabel('Default description', { exact: true })).toHaveValue('Browser saved site description.');
  const stale = await context.newPage();
  await stale.goto(origin + '/admin/seo');
  await page.getByLabel('Publisher name', { exact: true }).fill('Browser publisher');
  await page.getByRole('button', { name: 'Save SEO settings', exact: true }).click();
  await expect(page.getByRole('status')).toHaveText('SEO settings saved.');
  await stale.getByLabel('Publisher name', { exact: true }).fill('Stale publisher');
  await stale.getByRole('button', { name: 'Save SEO settings', exact: true }).click();
  await expect(stale.getByRole('alert')).toBeFocused();
  await expect(stale.getByLabel('Publisher name', { exact: true })).toHaveValue('Stale publisher');
  await stale.close();
  await page.getByLabel('Title format', { exact: true }).fill('{unknown}');
  await page.getByRole('button', { name: 'Save SEO settings', exact: true }).click();
  await expect(page.getByRole('alert')).toBeFocused();
  await expect(page.getByLabel('Title format', { exact: true })).toHaveAttribute('aria-invalid', 'true');
  await screenshot(page, 'seo-validation.png');
  await page.reload();
  await screenshot(page, 'seo-desktop-light.png');
  await page.getByRole('button', { name: 'Dark mode', exact: true }).focus();
  await page.keyboard.press('Space');
  await expect(page.locator('[data-shell="admin"]')).toHaveAttribute('data-theme', 'dark');
  await screenshot(page, 'seo-desktop-dark.png');
  await page.setViewportSize({ width: 320, height: 844 });
  for (const label of ['Site settings', 'Findings and previews', 'Redirects', '404 report']) {
    await page.getByRole('button', { name: label, exact: true }).click();
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${label} overflows mobile viewport`);
    await screenshot(page, `seo-mobile-${label.replaceAll(' ', '-')}.png`);
  }
  await expect(page.locator('.seo-missing-list')).toContainText('/missing-guide');
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.getByRole('button', { name: 'Redirects', exact: true }).click();
  await page.getByLabel('Old path', { exact: true }).fill('/browser-old');
  await page.getByLabel('Destination path', { exact: true }).fill('/articles/seo-renamed');
  await page.getByRole('button', { name: 'Save redirect', exact: true }).click();
  await expect(page.getByRole('status')).toHaveText('Redirect saved.');
  const redirect = await context.request.get(origin + '/browser-old?tracking=private', { maxRedirects: 0 });
  assert.equal(redirect.status(), 301);
  assert.equal(redirect.headers().location, '/articles/seo-renamed');
  await page.getByLabel('Old path', { exact: true }).fill('/browser-invalid');
  await page.getByLabel('Destination path', { exact: true }).fill('https://outside.test/');
  await page.getByRole('button', { name: 'Save redirect', exact: true }).click();
  await expect(page.getByRole('alert')).toBeFocused();
  await page.goto(origin + '/admin/seo?kind=article');
  const preview = page.locator('.seo-finding').filter({ has: page.getByRole('link', { name: '/articles/seo-renamed', exact: true }) });
  const previewTitle = await preview.locator('.seo-preview-title').first().textContent();
  const previewDescription = await preview.locator('.seo-preview-description').first().textContent();
  await preview.getByRole('link', { name: '/articles/seo-renamed', exact: true }).click();
  await head(page, '/articles/seo-renamed');
  await expect(page).toHaveTitle(previewTitle);
  await expect(page.locator('head meta[name="description"]')).toHaveAttribute('content', previewDescription);
  await expect(page.locator('head link[type="text/markdown"]')).toHaveAttribute('href', origin + '/articles/seo-renamed.md');
  await page.goto(origin + '/articles');
  await page.locator('a[href="/articles/seo-renamed"]').first().click();
  await head(page, '/articles/seo-renamed');
  await expect(page.locator('head script[type="application/ld+json"]')).toHaveCount(1);
  const schema = JSON.parse(await page.locator('head script[type="application/ld+json"]').textContent());
  assert.ok(JSON.stringify(schema).includes('Browser publisher'));
  const nojs = await browser.newContext({ javaScriptEnabled: false });
  const initial = await nojs.newPage();
  await initial.goto(origin + '/articles/seo-renamed');
  await head(initial, '/articles/seo-renamed');
  await expect(initial.locator('body')).toContainText('article café →');
  await screenshot(initial, 'seo-no-javascript.png');
  await nojs.close();
  for (const name of ['editor', 'manager']) {
    const delegated = await browser.newContext();
    const actor = await delegated.newPage();
    await login(actor, name, '/admin');
    const nav = actor.getByRole('navigation', { name: 'Administration', exact: true });
    await expect(nav.getByRole('link', { name: 'SEO', exact: true })).toHaveCount(name === 'manager' ? 1 : 0);
    const response = await actor.goto(origin + '/admin/seo');
    assert.equal(response.status(), name === 'manager' ? 200 : 403);
    await delegated.close();
  }
  const editorContext = await browser.newContext();
  const editor = await editorContext.newPage();
  await login(editor, 'editor', '/admin/articles?state=published');
  await editor.locator('.editorial-admin-list > li').getByRole('link').click();
  await editor.getByLabel('Search title', { exact: true }).fill('Browser private article SEO');
  await editor.getByLabel('Search description', { exact: true }).fill('Browser private description.');
  await editor.getByRole('checkbox', { name: 'Exclude this page from search engines', exact: true }).check();
  await editor.getByRole('button', { name: 'Save draft', exact: true }).click();
  await expect(editor.getByRole('status')).toHaveText('Changes saved.');
  await editor.reload();
  await expect(editor.getByLabel('Search title', { exact: true })).toHaveValue('Browser private article SEO');
  await page.goto(origin + '/articles/seo-renamed');
  await expect(page).toHaveTitle(previewTitle);
  await head(page, '/articles/seo-renamed');
  await editor.getByRole('button', { name: 'Publish saved changes', exact: true }).click();
  await expect(editor.getByRole('status')).toHaveText('Changes saved.');
  await page.reload();
  await expect(page).toHaveTitle('Browser private article SEO · Directory');
  await expect(page.locator('head meta[name="robots"]')).toHaveAttribute('content', 'noindex, follow');
  await editorContext.close();
  const ownerContext = await browser.newContext();
  const owner = await ownerContext.newPage();
  await login(owner, 'owner', '/dashboard/listings');
  await owner.getByRole('link', { name: 'Edit listing', exact: true }).click();
  await owner.getByLabel('Search title', { exact: true }).fill('Browser private listing SEO');
  await owner.getByRole('button', { name: 'Save draft', exact: true }).click();
  await expect(owner.getByRole('status')).toHaveText('Draft saved.');
  await owner.reload();
  await expect(owner.getByLabel('Search title', { exact: true })).toHaveValue('Browser private listing SEO');
  await page.goto(origin + '/listings');
  await page.getByRole('link', { name: 'SEO listing', exact: true }).click();
  await expect(page).toHaveTitle('Shared search title · Directory');
  await ownerContext.close();
  assert.deepEqual(errors, []);
  console.log('SEO settings persistence, stale/invalid inputs, keyboard/mobile themes, redirects, previews, SSR and Inertia metadata, Markdown discovery and delegated permissions passed.');
} catch (error) {
  if (browser) for (const context of browser.contexts()) for (const page of context.pages()) {
    try { await screenshot(page, 'seo-failure.png'); }
    catch (captureError) { console.error('Failure screenshot unavailable:', captureError.message); }
  }
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
