import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run adoption journeys with the real SSR renderer');
assert.ok(process.env.CARGO_TARGET_DIR, 'CARGO_TARGET_DIR is required');
const probe = createServer();
await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_NAME: 'Adoption Catalog', SITE_DESCRIPTION: 'A configured directory for adoption.', SITE_ACCENT: '#673ab7', SITE_LOGO_URL: 'https://brand.example.test/logo.svg', APP_ENV: 'production', APP_URL: origin, SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
let server, browser;
let logs = '';
const errors = [];
async function stop() {
  if (!server || server.exitCode !== null || server.signalCode !== null) return;
  server.kill('SIGTERM');
  await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(6000)]);
  if (server.exitCode === null && server.signalCode === null) server.kill('SIGKILL');
}
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
function watchPage(page) {
  page.on('pageerror', error => errors.push(error.message));
  page.on('dialog', dialog => { errors.push(`Unexpected dialog: ${dialog.message()}`); void dialog.dismiss(); });
}
async function login(page, email) {
  await page.goto(origin + '/login');
  await page.getByLabel('Email address', { exact: true }).fill(email);
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(origin + '/dashboard');
}

async function screenshot(page, name) {
  if (!env.ADOPTION_ARTIFACT_DIR) return;
  await mkdir(env.ADOPTION_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.ADOPTION_ARTIFACT_DIR, name), fullPage: true });
}
async function branding(page, shell) {
  await expect(page.locator(`[data-shell="${shell}"]`)).toBeVisible();
  await expect(page.getByRole('link', { name: 'Adoption Catalog home', exact: true })).toBeVisible();
  await expect(page.locator('.site-brand')).toContainText(env.APP_NAME);
  await expect(page.locator('.site-brand img')).toHaveAttribute('src', env.SITE_LOGO_URL);
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', env.SITE_ACCENT);
  assert.equal(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--site-accent').trim()), env.SITE_ACCENT);
}
async function fits(page, label) {
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${label} overflows`);
}
const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1000 } }),
    browser.newContext({ viewport: { width: 1440, height: 1000 } }),
    browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } }),
  ]);
  for (const context of contexts) await context.route(env.SITE_LOGO_URL, route => route.fulfill({
    status: 200, contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#673ab7"/></svg>',
  }));
  const [admin, member, noJs] = await Promise.all(contexts.map(context => context.newPage()));
  for (const page of [admin, member, noJs]) watchPage(page);

  // This check cannot be satisfied by hydration: JavaScript is disabled and the
  // independent raw HTTP request must contain the identity and metadata too.
  const response = await fetch(origin + '/');
  assert.equal(response.status, 200);
  const html = await response.text();
  assert.ok(html.includes('Adoption Catalog'), 'Initial HTML lacks configured name');
  assert.ok(html.includes(env.SITE_DESCRIPTION), 'Initial HTML lacks configured description');
  assert.ok(html.includes(env.SITE_LOGO_URL), 'Initial HTML lacks configured logo');
  await noJs.goto(origin + '/');
  await branding(noJs, 'public');
  await expect(noJs.locator('meta[name="description"]')).toHaveAttribute('content', env.SITE_DESCRIPTION);
  await expect(noJs.locator('link[rel="canonical"]')).toHaveAttribute('href', origin + '/');
  await expect(noJs.locator('meta[property="og:title"]')).toHaveAttribute('content', /Adoption Catalog/);
  await expect(noJs.locator('meta[property="og:url"]')).toHaveAttribute('content', origin + '/');
  await expect(noJs).toHaveTitle(/Adoption Catalog/);
  await expect(noJs.locator('.site-footer')).toContainText(env.SITE_DESCRIPTION);
  await fits(noJs, 'Public mobile without JavaScript');
  await screenshot(noJs, 'adoption-public-mobile-no-js.png');

  await login(admin, 'adoption-browser-admin@example.test');
  await admin.goto(origin + '/admin');
  await branding(admin, 'admin');
  await expect(admin.locator('meta[name="robots"]')).toHaveAttribute('content', /noindex/);
  await fits(admin, 'Administrator desktop');
  await screenshot(admin, 'adoption-admin-desktop.png');
  await admin.setViewportSize({ width: 390, height: 844 });
  await fits(admin, 'Administrator mobile');
  await screenshot(admin, 'adoption-admin-mobile.png');

  await login(member, 'member@example.test');
  await member.goto(origin + '/dashboard/listings');
  await member.getByRole('link', { name: 'Adoption notification listing', exact: true }).click();
  await expect(member.getByRole('heading', { name: 'Edit listing', exact: true })).toBeVisible();
  await branding(member, 'public');
  const updates = member.getByRole('region', { name: 'Updates for this listing', exact: true });
  await expect(updates).toBeVisible();
  await expect(updates.getByRole('heading', { name: 'Listing approved', exact: true })).toBeVisible();
  await expect(updates.getByRole('heading', { name: 'Listing rejected', exact: true })).toBeVisible();
  const escapedReason = '<script>window.adoptionInjected=true</script>';
  await expect(updates).toContainText(escapedReason);
  assert.equal(await updates.locator('script').count(), 0, 'Notification markup must be escaped');
  assert.equal(await member.evaluate(() => Boolean(window.adoptionInjected)), false);
  assert.ok(await updates.locator('li').count() <= 20, 'Owner history must be bounded');
  const beforeRestart = await updates.innerText();
  const ownerUrl = member.url();
  await screenshot(member, 'adoption-owner-updates-desktop.png');
  await stop();
  await start();
  await member.goto(ownerUrl);
  await expect(updates).toHaveText(beforeRestart, { useInnerText: true });
  await member.setViewportSize({ width: 390, height: 844 });
  await fits(member, 'Owner history mobile');
  await screenshot(member, 'adoption-owner-updates-mobile.png');
  expect(errors).toEqual([]);
  console.log('Configured public/admin branding, initial metadata without JavaScript, escaped durable owner notification history and mobile journeys passed.');
} catch (error) {
  if (browser && env.ADOPTION_ARTIFACT_DIR) {
    let index = 0;
    for (const context of browser.contexts()) for (const page of context.pages()) {
      try { await screenshot(page, `adoption-failure-${index++}.png`); }
      catch (captureError) { console.error('Failure screenshot unavailable:', captureError.message); }
    }
  }
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
