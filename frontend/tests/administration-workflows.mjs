import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run administration journeys with the real SSR renderer');
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
async function save(page, keyboard = false) {
  const button = page.getByRole('button', { name: 'Save access', exact: true });
  if (keyboard) { await button.focus(); await page.keyboard.press('Enter'); }
  else await button.click();
  await expect(page.getByRole('status')).toContainText('Account access saved.');
  await expect(button).toBeEnabled();
}
async function screenshot(page, name) {
  if (!env.ADMINISTRATION_ARTIFACT_DIR) return;
  await mkdir(env.ADMINISTRATION_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.ADMINISTRATION_ARTIFACT_DIR, name), fullPage: true });
}
async function fits(page, label) {
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${label} overflows the viewport`);
}
const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const adminContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const memberContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const admin = await adminContext.newPage();
  const member = await memberContext.newPage();
  watchPage(admin); watchPage(member);
  await login(admin, 'administration-browser-admin@example.test');
  await login(member, 'administration-browser-member@example.test');
  await admin.goto(origin + '/admin/accounts');
  await admin.getByLabel('Search accounts', { exact: true }).fill('administration-browser-member@example.test');
  await admin.getByRole('button', { name: 'Search', exact: true }).click();
  const row = admin.locator('.editorial-admin-list > li').filter({ hasText: 'administration-browser-member@example.test' });
  await expect(row).toHaveCount(1);
  await row.getByRole('link').click();
  await expect(admin.getByRole('heading', { name: 'Account access', exact: true })).toBeVisible();
  const accountUrl = admin.url();
  assert.match(accountUrl, /\/admin\/accounts\/\d+$/);
  const editor = admin.locator('input[name="roles"][value="editor"]');
  await expect(editor).not.toBeChecked();
  await editor.focus(); await admin.keyboard.press('Space');
  await expect(editor).toBeChecked();
  await save(admin, true);

  // An already authenticated member receives the changed capability on their next request.
  await member.goto(origin + '/admin');
  const memberNav = member.getByRole('navigation', { name: 'Administration', exact: true });
  await expect(memberNav.getByRole('link', { name: 'Articles', exact: true })).toBeVisible();
  for (const name of ['Accounts', 'Payment providers', 'Publishing plans', 'Categories and tags', 'Audit history']) {
    await expect(memberNav.getByRole('link', { name, exact: true })).toHaveCount(0);
  }
  await memberNav.getByRole('link', { name: 'Articles', exact: true }).click();
  await expect(member.getByRole('heading', { name: 'Articles', exact: true })).toBeVisible();

  await admin.getByRole('checkbox', { name: 'Suspend account', exact: true }).check();
  await save(admin);
  await member.goto(origin + '/dashboard');
  await expect(member).toHaveURL(/\/login(?:\?|$)/);
  await expect(member.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
  await admin.getByRole('checkbox', { name: 'Suspend account', exact: true }).uncheck();
  await save(admin);
  await login(member, 'administration-browser-member@example.test');

  // The second editor must retain their selections after a stale version fails.
  const stale = await adminContext.newPage();
  watchPage(stale);
  await stale.goto(accountUrl);
  await admin.locator('input[name="roles"][value="moderator"]').check();
  await save(admin);
  await stale.locator('input[name="roles"][value="editor"]').uncheck();
  await stale.getByRole('button', { name: 'Save access', exact: true }).click();
  const alert = stale.getByRole('alert').filter({ hasText: 'Access was not saved' });
  await expect(alert).toBeVisible();
  await expect(alert).toBeFocused();
  await expect(alert.getByRole('link', { name: 'Reload latest access', exact: true })).toBeVisible();
  await expect(stale.locator('input[name="roles"][value="editor"]')).not.toBeChecked();
  await expect(stale.locator('input[name="roles"][value="moderator"]')).not.toBeChecked();
  await screenshot(stale, 'account-stale-access-desktop.png');
  await stale.setViewportSize({ width: 390, height: 844 });
  await fits(stale, 'Account detail');
  await screenshot(stale, 'account-detail-mobile.png');
  await stale.close();

  await admin.goto(origin + '/admin/audit');
  await expect(admin.getByRole('heading', { name: 'Audit history', exact: true })).toBeVisible();
  const entries = admin.locator('.editorial-admin-list > li');
  await expect(entries.first()).toBeVisible();
  await expect(entries.filter({ hasText: /\buser\b/ }).first()).toBeVisible();
  // Host bootstrap records can precede enough user changes to appear on an older page.
  let hostFound = await entries.filter({ hasText: 'Host operator' }).count() > 0;
  let pages = 0;
  while (!hostFound && await admin.getByRole('link', { name: 'Next', exact: true }).count()) {
    assert.ok(++pages <= 20, 'Host audit fixture was not found within 20 bounded pages');
    const previousUrl = admin.url();
    await admin.getByRole('link', { name: 'Next', exact: true }).click();
    await expect(admin).not.toHaveURL(previousUrl);
    hostFound = await entries.filter({ hasText: 'Host operator' }).count() > 0;
  }
  assert.ok(hostFound, 'Host administrative action must appear in audit history');
  await fits(admin, 'Desktop audit history');
  await screenshot(admin, 'audit-desktop.png');
  await admin.setViewportSize({ width: 390, height: 844 });
  await fits(admin, 'Mobile audit history');
  await screenshot(admin, 'audit-mobile.png');
  await admin.goto(origin + '/admin/accounts?q=administration-browser');
  await fits(admin, 'Account search');
  await screenshot(admin, 'accounts-mobile.png');
  expect(errors).toEqual([]);
  console.log('Account search, keyboard role grants, live session capabilities, suspension/reinstatement, stale access recovery, host/user audit history and mobile journeys passed.');
} catch (error) {
  if (browser && env.ADMINISTRATION_ARTIFACT_DIR) {
    let index = 0;
    for (const context of browser.contexts()) for (const page of context.pages()) {
      try { await screenshot(page, `administration-failure-${index++}.png`); }
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
