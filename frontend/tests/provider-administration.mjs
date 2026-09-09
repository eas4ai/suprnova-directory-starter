import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const portProbe = createServer();
await new Promise(resolve => portProbe.listen(0, '127.0.0.1', resolve));
const port = portProbe.address().port;
await new Promise(resolve => portProbe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin, SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
const markers = ['BROWSER_STRIPE_SECRET', 'BROWSER_STRIPE_WEBHOOK', 'BROWSER_PADDLE_SECRET', 'BROWSER_PADDLE_WEBHOOK'];
const server = spawn(join(env.CARGO_TARGET_DIR, 'debug/directory'), ['serve', '--no-migrate'], { env, stdio: ['ignore', 'pipe', 'pipe'] });
let logs = '';
server.stdout.on('data', data => { logs = (logs + data).slice(-16000); });
server.stderr.on('data', data => { logs = (logs + data).slice(-16000); });
let browser;
const watchdog = setTimeout(() => { server.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 120_000);
try {
  let ready = false;
  for (let i = 0; i < 150; i++) {
    if (server.exitCode !== null) throw new Error(`Application exited: ${logs}`);
    try { ready = (await fetch(`${origin}/_suprnova/health`, { signal: AbortSignal.timeout(500) })).ok; } catch {}
    if (ready) break;
    await delay(100);
  }
  assert.ok(ready, `Application did not become ready: ${logs}`);
  browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const errors = [];
  const responses = [];
  const responseErrors = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => {
    if (response.url().startsWith(`${origin}/admin/billing`) && (response.status() < 300 || response.status() >= 400)) responses.push(response.text().then(body => {
      for (const marker of markers) assert.ok(!body.includes(marker), 'A billing response leaked a replacement credential');
    }).catch(error => responseErrors.push(error.message)));
  });
  async function login() {
    await page.goto(`${origin}/login`);
    await page.getByLabel('Email address', { exact: true }).fill('billing-operator@example.test');
    await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page).toHaveURL(`${origin}/dashboard`);
  }
  async function tabTo(locator) {
    for (let i = 0; i < 70; i++) {
      if (await locator.evaluate(el => el === document.activeElement)) return;
      await page.keyboard.press('Tab');
    }
    throw new Error('Billing control is not reachable by keyboard');
  }
  await login();
  await page.getByRole('link', { name: 'Administration', exact: true }).click();
  await page.getByRole('link', { name: 'Payment providers', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Payment providers', exact: true })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Test mode', exact: true })).toHaveAttribute('aria-current', 'page');
  await page.getByLabel('Stripe API key', { exact: true }).fill('sk_test_BROWSER_STRIPE_SECRET');
  await page.getByLabel('Stripe publishable key', { exact: true }).fill('pk_test_browser');
  await page.getByLabel('Stripe webhook key', { exact: true }).fill('whsec_BROWSER_STRIPE_WEBHOOK');
  await page.getByLabel('Enable Stripe in test mode', { exact: true }).check();
  await page.getByRole('button', { name: 'Show Stripe API key', exact: true }).click();
  await expect(page.getByLabel('Stripe API key', { exact: true })).toHaveAttribute('type', 'text');
  await page.getByRole('button', { name: 'Hide Stripe API key', exact: true }).click();
  await page.getByRole('radio', { name: 'Stripe', exact: true }).check();
  await page.getByRole('button', { name: 'Add plan', exact: true }).click();
  const lastPlan = page.getByLabel('Plan identifier', { exact: true }).last();
  await expect(lastPlan).toBeFocused();
  await lastPlan.fill('standard');
  await page.getByLabel('Stripe price (optional)', { exact: true }).last().fill('price_test_standard');
  await page.getByLabel('Paddle price (optional)', { exact: true }).last().fill('pri_test_standard');
  const saveTest = page.getByRole('button', { name: 'Save test settings', exact: true });
  await tabTo(saveTest);
  await page.keyboard.press('Enter');
  await expect(page.getByRole('status')).toContainText('Settings saved.');
  await expect(page.getByLabel('Stripe API key', { exact: true })).toHaveValue('');
  await expect(page.getByLabel('Stripe webhook key', { exact: true })).toHaveValue('');
  await page.reload();
  await expect(lastPlan).toHaveValue('standard');
  await expect(page.getByRole('radio', { name: 'Stripe', exact: true })).toBeChecked();
  if (env.PROVIDER_ARTIFACT_DIR) {
    await mkdir(env.PROVIDER_ARTIFACT_DIR, { recursive: true });
    await page.screenshot({ path: join(env.PROVIDER_ARTIFACT_DIR, 'desktop.png'), fullPage: true });
  }

  await page.getByLabel('Stripe API key', { exact: true }).fill('sk_live_invalid_for_test');
  await saveTest.click();
  const error = page.getByRole('alert');
  await expect(error).toContainText('Changes were not saved');
  await expect(error).toContainText('sk_test_');
  await expect(error).toBeFocused();
  await expect(page.getByLabel('Stripe API key', { exact: true })).toHaveValue('sk_live_invalid_for_test');
  page.once('dialog', dialog => dialog.dismiss());
  await page.getByRole('link', { name: 'Live mode', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Test mode', exact: true })).toHaveAttribute('aria-current', 'page');
  page.once('dialog', dialog => dialog.accept());
  await page.getByRole('link', { name: 'Live mode', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Live mode', exact: true })).toHaveAttribute('aria-current', 'page');
  await expect(page.getByLabel('Plan identifier', { exact: true })).toHaveCount(1);
  await page.getByLabel('Paddle API key', { exact: true }).fill('pdl_live_apikey_BROWSER_PADDLE_SECRET');
  await page.getByLabel('Paddle client token', { exact: true }).fill('live_browser');
  await page.getByLabel('Paddle webhook key', { exact: true }).fill('pdl_ntfset_BROWSER_PADDLE_WEBHOOK');
  await page.getByRole('radio', { name: 'Paddle', exact: true }).check();
  await page.getByRole('button', { name: 'Save live settings', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Settings saved.');
  await expect(page.getByLabel('Paddle API key', { exact: true })).toHaveValue('');
  await expect(page.getByRole('radio', { name: 'Paddle', exact: true })).toBeChecked();

  // A second browser page saves first. The stale editor must keep its input and
  // show an explicit conflict instead of silently overwriting the new version.
  const second = await page.context().newPage();
  await second.goto(`${origin}/admin/billing?mode=live`);
  await second.getByLabel('Stripe publishable key', { exact: true }).fill('pk_live_second_editor');
  await second.getByRole('button', { name: 'Save live settings', exact: true }).click();
  await expect(second.getByRole('status')).toContainText('Settings saved.');
  await page.getByLabel('Stripe publishable key', { exact: true }).fill('pk_live_stale_editor');
  await page.getByRole('button', { name: 'Save live settings', exact: true }).click();
  await expect(error).toContainText('Another administrator saved changes');
  await expect(page.getByLabel('Stripe publishable key', { exact: true })).toHaveValue('pk_live_stale_editor');
  page.once('dialog', dialog => dialog.accept());
  await page.getByRole('link', { name: 'Reload saved settings', exact: true }).click();
  await expect(page.getByLabel('Stripe publishable key', { exact: true })).toHaveValue('pk_live_second_editor');
  await second.close();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByLabel('Enable Paddle in live mode', { exact: true }).uncheck();
  await expect(page.getByRole('radio', { name: 'None', exact: true })).toBeChecked();
  await page.getByLabel('Clear Paddle credentials when saved', { exact: true }).check();
  await page.getByRole('button', { name: 'Remove plan featured', exact: true }).click();
  await page.getByRole('button', { name: 'Save live settings', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Settings saved.');
  await expect(page.getByLabel('Paddle client token', { exact: true })).toHaveValue('');
  await expect(page.getByText('No plans mapped in live mode.', { exact: false })).toBeVisible();
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), 'Billing page overflows on mobile');
  if (env.PROVIDER_ARTIFACT_DIR) {
    await page.locator('#main-content').focus();
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.screenshot({ path: join(env.PROVIDER_ARTIFACT_DIR, 'mobile.png'), fullPage: true });
  }

  const revoke = spawnSync(join(env.CARGO_TARGET_DIR, 'debug/console'), ['admin:access', 'revoke', '--user-id', env.BILLING_OPERATOR_ID], { env, encoding: 'utf8', timeout: 15_000 });
  assert.equal(revoke.status, 0, revoke.stderr);
  await page.reload();
  await expect(page).toHaveURL(`${origin}/dashboard`);
  await expect(page.locator('[data-shell="admin"]')).toHaveCount(0);
  await Promise.all(responses);
  expect(responseErrors).toEqual([]);
  for (const marker of markers) assert.ok(!logs.includes(marker), 'The server log leaked a credential');
  expect(errors).toEqual([]);
  console.log('Both provider forms, validation, secret clearing, mode isolation, keyboard save, stale-edit conflict, mobile layout and CLI revocation passed.');
} catch (error) {
  console.error(logs);
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
