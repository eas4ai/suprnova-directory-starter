import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { mkdir } from 'node:fs/promises';
import { createServer } from 'node:net';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const portProbe = createServer();
await new Promise(resolve => portProbe.listen(0, '127.0.0.1', resolve));
const port = portProbe.address().port;
await new Promise(resolve => portProbe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const server = spawn(join(process.env.CARGO_TARGET_DIR, 'debug/directory'), ['serve', '--no-migrate'], {
  env: { ...process.env, APP_ENV: 'production', APP_URL: origin,
    APP_KEY: randomBytes(32).toString('base64url'), SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
    // This isolated UI fixture is one process and sends no account mail.
    MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true',
    RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' },
  stdio: ['ignore', 'pipe', 'pipe'],
});
let logs = '';
server.stdout.on('data', data => { logs = (logs + data).slice(-12000); });
server.stderr.on('data', data => { logs = (logs + data).slice(-12000); });
let browser;
const watchdog = setTimeout(() => {
  server.kill('SIGKILL');
  void browser?.close();
  process.exitCode = 1;
}, 90_000);
try {
  let ready = false;
  for (let i = 0; i < 100; i++) {
    if (server.exitCode !== null) throw new Error(`Application exited: ${logs}`);
    try { ready = (await fetch(`${origin}/_suprnova/health`, { signal: AbortSignal.timeout(500) })).ok; } catch {}
    if (ready) break;
    await delay(100);
  }
  if (!ready) throw new Error(`Application did not become ready: ${logs}`);
  browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(origin);
  await expect(page.locator('[data-shell="public"]')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'A place for good discoveries.' })).toBeVisible();
  const brand = page.locator('[data-brand-mark]').first();
  const publicColor = await brand.evaluate(el => getComputedStyle(el).backgroundColor);
  const artifacts = process.env.FOUNDATION_ARTIFACT_DIR;
  await mkdir(artifacts, { recursive: true });
  await page.screenshot({ path: join(artifacts, 'public.png'), fullPage: true });

  async function tabTo(locator) {
    for (let i = 0; i < 30; i++) {
      await page.keyboard.press('Tab');
      if (await locator.evaluate(el => el === document.activeElement)) return;
    }
    throw new Error('Control was not reachable by keyboard');
  }
  await page.setViewportSize({ width: 390, height: 844 });
  const menu = page.getByRole('button', { name: 'Open navigation' });
  await tabTo(menu);
  await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog', { name: 'Navigation' });
  await expect(dialog).toBeVisible();
  for (let i = 0; i < 8; i++) {
    await page.keyboard.press('Tab');
    const focus = await dialog.evaluate(el => ({ inside: el.contains(document.activeElement),
      browserChrome: !document.hasFocus() && document.activeElement === document.body, open: el.open }));
    // Native dialogs allow the browser toolbar, but never background page controls.
    expect(focus.open && (focus.inside || focus.browserChrome), JSON.stringify({ step: i, ...focus })).toBe(true);
  }
  await page.keyboard.press('Escape');
  await expect(dialog).not.toBeVisible();
  await expect(menu).toBeFocused();
  await page.screenshot({ path: join(artifacts, 'mobile.png'), fullPage: true });
  await page.setViewportSize({ width: 1440, height: 900 });

  async function login(email, password) {
    await page.goto(`${origin}/login`);
    await page.getByLabel('Email address', { exact: true }).fill(email);
    await page.getByLabel('Password', { exact: true }).fill(password);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page).toHaveURL(`${origin}/dashboard`);
  }
  await login('member@example.test', 'member-password-123');
  await page.goto(`${origin}/admin`);
  await expect(page).toHaveURL(`${origin}/dashboard`);
  await expect(page.locator('[data-shell="admin"]')).toHaveCount(0);
  await page.context().clearCookies();
  await login('operator@example.test', 'operator-password-123');
  await tabTo(page.getByRole('link', { name: 'Administration', exact: true }));
  await page.keyboard.press('Enter');
  await expect(page.locator('[data-shell="admin"]')).toBeVisible();
  expect(await brand.evaluate(el => getComputedStyle(el).backgroundColor)).toBe(publicColor);
  await page.screenshot({ path: join(artifacts, 'admin.png'), fullPage: true });
  await page.evaluate(() => document.documentElement.style.setProperty('--brand-accent', '#9b2c2c'));
  expect(await brand.evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(155, 44, 44)');
  await page.getByRole('link', { name: 'View directory', exact: true }).click();
  await expect(page.locator('[data-shell="public"]')).toBeVisible();
  expect(await brand.evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(155, 44, 44)');
  await page.getByRole('link', { name: 'Administration', exact: true }).click();
  await expect(page.locator('[data-shell="admin"]')).toBeVisible();
  await expect(page).toHaveURL(`${origin}/admin`);
  const signOut = page.getByRole('button', { name: 'Sign out', exact: true });
  await tabTo(signOut);
  await page.keyboard.press('Enter');
  const signOutDialog = page.getByRole('dialog', { name: 'Sign out of your account?' });
  await expect(signOutDialog).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(signOutDialog).not.toBeVisible();
  // Let native close events and the component's focus restoration settle.
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  await expect(signOut).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(signOutDialog).toBeVisible();
  await tabTo(signOutDialog.getByRole('button', { name: 'Sign out now' }));
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(`${origin}/`);
  await expect(page.locator('[data-shell="public"]')).toBeVisible();
  await login('member@example.test', 'member-password-123');
  await page.setViewportSize({ width: 390, height: 844 });
  await tabTo(menu);
  await page.keyboard.press('Enter');
  await expect(dialog).toBeVisible();
  await tabTo(dialog.getByRole('button', { name: 'Sign out', exact: true }));
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(`${origin}/`);
  await page.goto(`${origin}/dashboard`);
  await expect(page).toHaveURL(`${origin}/login`);
  expect(errors).toEqual([]);
  console.log('Public/admin rendering, permission denial, keyboard dialogs and shared branding passed.');
} catch (error) {
  console.error(logs);
    if (browser) for (const context of browser.contexts()) for (const page of context.pages()) console.error(await page.locator('iframe').count() ? await page.frames().at(-1).locator('body').innerText() : 'No error iframe');
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  if (server.exitCode === null && server.signalCode === null) {
    server.kill('SIGTERM');
    await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(6000)]);
    if (server.exitCode === null && server.signalCode === null) server.kill('SIGKILL');
  }
}
