import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const probe = createServer();
await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin, PAID_BROWSER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
let server, browser, logs = '';
async function stop() {
  if (!server || server.exitCode !== null) return;
  try { process.kill(-server.pid, 'SIGTERM'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
  await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(3000)]);
  try { process.kill(-server.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
}
const watchdog = setTimeout(() => { void stop(); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  server = spawn('cargo', ['test', '--locked', '--test', 'paid_lifecycle', 'serve_paid_browser_fixture', '--', '--ignored', '--nocapture'], { env, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
  for (const stream of [server.stdout, server.stderr]) stream.on('data', data => { logs = (logs + data).slice(-20000); });
  for (let n = 0; !logs.includes('PAID_BROWSER_READY'); n++) {
    if (n > 600 || server.exitCode !== null) throw new Error(`Fixture server failed: ${logs}`);
    await delay(100);
  }
  browser = await chromium.launch();
  const ownerContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const adminContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  ownerContext.setDefaultTimeout(10000);
  adminContext.setDefaultTimeout(10000);
  const owner = await ownerContext.newPage();
  const admin = await adminContext.newPage();
  const visitor = await browser.newPage();
  const errors = [];
  for (const page of [owner, admin, visitor]) page.on('pageerror', e => errors.push(e.message));
  async function login(page, name) {
    await page.goto(`${origin}/login`);
    assert.ok((await page.content()).includes('data-page'), `Missing Inertia page: ${await page.content()}`);
    await page.getByLabel('Email address', { exact: true }).fill(`${name}@example.test`);
    await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page).toHaveURL(`${origin}/dashboard`);
  }
  await login(admin, 'browser-admin');
  await admin.goto(`${origin}/admin/plans`);
  await expect(admin.getByRole('heading', { name: 'Publishing plans', exact: true })).toBeVisible();
  await admin.getByLabel('Plan identifier', { exact: true }).fill('browser-plan');
  await admin.getByLabel('Name', { exact: true }).fill('Browser plan');
  await admin.getByLabel('Billing type', { exact: true }).selectOption('monthly');
  await admin.getByLabel('Amount in minor units', { exact: true }).fill('0');
  await admin.getByRole('button', { name: 'Create plan', exact: true }).click();
  await expect(admin.getByRole('alert')).toContainText('Check your plan');
  await expect(admin.getByRole('alert')).toBeFocused();
  await admin.getByLabel('Amount in minor units', { exact: true }).fill('2500');
  await admin.getByRole('button', { name: 'Create plan', exact: true }).click();
  await expect(admin.getByRole('status')).toContainText('saved');
  await admin.getByRole('button', { name: /Browser plan/ }).click();
  await expect(admin.getByLabel('Plan identifier', { exact: true })).toHaveAttribute('readonly', '');
  await admin.getByLabel('Name', { exact: true }).fill('Browser plan edited');
  await admin.getByLabel('Offer this plan to owners', { exact: true }).uncheck();
  await admin.getByRole('button', { name: 'Save plan', exact: true }).click();
  await expect(admin.getByRole('button', { name: /Browser plan edited/ })).toContainText('Disabled');
  if (env.PAID_ARTIFACT_DIR) { await mkdir(env.PAID_ARTIFACT_DIR, { recursive: true }); await admin.screenshot({ path: join(env.PAID_ARTIFACT_DIR, 'publishing-plans-desktop.png'), fullPage: true }); }

  await login(owner, 'browser-owner');
  await owner.goto(`${origin}/dashboard/listings/create`);
  await owner.getByLabel('Title', { exact: true }).fill('Browser Stripe resource');
  await owner.getByLabel('Summary', { exact: true }).fill('A publishing checkout browser journey.');
  await owner.getByLabel('Description', { exact: true }).fill('Reviewed publishing resource.');
  await owner.getByLabel('Website URL', { exact: true }).fill('https://example.test/resource');
  await owner.getByRole('checkbox', { name: 'Software', exact: true }).check();
  await owner.getByRole('button', { name: 'Save draft', exact: true }).click();
  await expect(owner.getByRole('status')).toContainText('Draft saved.');
  const id = owner.url().match(/\/listings\/(\d+)\/edit$/)?.[1];
  assert.ok(id);
  await owner.getByRole('button', { name: 'Submit for review', exact: true }).click();
  await expect(owner.getByText('This revision is awaiting review.', { exact: false })).toBeVisible();
  await admin.goto(`${origin}/admin/listings/${id}`);
  await admin.getByRole('radio', { name: 'Approve revision', exact: true }).check();
  await admin.getByRole('button', { name: 'Record decision', exact: true }).click();
  await expect(admin.getByRole('heading', { name: 'Current revision · approved', exact: true })).toBeVisible();
  await owner.goto(`${origin}/dashboard/listings/${id}/plans`);
  const once = owner.getByRole('form', { name: 'once publishing', exact: true });
  await once.getByLabel('Payment provider', { exact: true }).selectOption('stripe');
  await once.getByRole('button', { name: 'Continue to checkout', exact: true }).click();
  await expect(owner.getByRole('heading', { name: 'Ready for payment', exact: true })).toBeVisible();
  const purchaseUrl = owner.url();
  await owner.goto(`${purchaseUrl}?paid=true&session_id=forged`);
  await expect(owner.getByRole('heading', { name: 'Ready for payment', exact: true })).toBeVisible();
  await visitor.goto(`${origin}/listings?q=Browser%20Stripe`);
  await expect(visitor.getByRole('heading', { name: 'No matching listings', exact: true })).toBeVisible();
  await owner.route('https://checkout.stripe.com/**', route => route.fulfill({ status: 302, headers: { location: `${origin}/__fixture/checkout/${route.request().url().split('/').pop()}` } }));
  await owner.getByRole('link', { name: 'Open Stripe checkout', exact: true }).click();
  await owner.getByRole('button', { name: 'Complete fixture payment', exact: true }).click();
  await expect(owner.getByRole('heading', { name: 'Active', exact: true })).toBeVisible();
  await expect(owner.getByText('Payment verified', { exact: true })).toBeVisible();
  await visitor.reload();
  await expect(visitor.getByRole('link', { name: 'Browser Stripe resource', exact: true })).toBeVisible();

  await owner.route('https://cdn.paddle.com/paddle/v2/paddle.js', route => route.fulfill({ contentType: 'application/javascript', body: `
    let callback; window.fixturePaddle = { initialize: 0, environment: 'production' };
    window.Paddle = { Environment: { set(e) { window.fixturePaddle.environment = e; } },
      Initialize(o) { window.fixturePaddle.initialize++; window.fixturePaddle.token = o.token; callback = o.eventCallback; },
      Update(o) { callback = o.eventCallback; }, Checkout: {
        async open(o) { window.fixturePaddle.transaction = o.transactionId; callback({name:'checkout.loaded'});
          const r = await fetch('/__fixture/settle/' + o.transactionId, {method:'POST'});
          if (!r.ok) throw new Error('Fixture settlement failed'); callback({name:'checkout.completed'}); },
        close() { callback({name:'checkout.closed'}); }
      } };` }));
  await owner.goto(`${origin}/dashboard/listings`);
  const paddleRow = owner.locator('.directory-owner-row').filter({ hasText: 'Browser Paddle resource' });
  await paddleRow.getByRole('link', { name: 'Choose publishing plan', exact: true }).click();
  const monthly = owner.getByRole('form', { name: 'monthly publishing', exact: true });
  await monthly.getByLabel('Payment provider', { exact: true }).selectOption('paddle');
  await monthly.getByRole('button', { name: 'Continue to checkout', exact: true }).click();
  await owner.getByRole('button', { name: 'Open Paddle checkout', exact: true }).click();
  await expect(owner.getByRole('heading', { name: 'Active', exact: true })).toBeVisible();
  const paddle = await owner.evaluate(() => window.fixturePaddle);
  assert.equal(paddle.initialize, 1); assert.equal(paddle.environment, 'production'); assert.match(paddle.transaction, /^txn_/); assert.match(paddle.token, /^live_/);
  await owner.getByRole('button', { name: 'Cancel renewal', exact: true }).click();
  await expect(owner.getByRole('group', { name: 'Cancel future renewals?', exact: true })).toBeFocused();
  await owner.getByRole('button', { name: 'Keep renewal', exact: true }).click();
  await expect(owner.getByRole('button', { name: 'Cancel renewal', exact: true })).toBeFocused();
  await owner.getByRole('button', { name: 'Cancel renewal', exact: true }).click();
  await owner.getByRole('button', { name: 'Confirm cancellation', exact: true }).click();
  await expect(owner.getByText('Cancellation was requested. We are waiting for provider confirmation.', { exact: true })).toBeVisible();
  assert.ok((await fetch(`${origin}/__fixture/reconcile`, { method: 'POST' })).ok);
  await owner.getByRole('button', { name: 'Refresh status', exact: true }).click();
  await expect(owner.getByText('Cancellation is scheduled. Your already-paid period is retained.', { exact: true })).toBeVisible();
  await owner.setViewportSize({ width: 390, height: 844 });
  assert.ok(await owner.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  if (env.PAID_ARTIFACT_DIR) await owner.screenshot({ path: join(env.PAID_ARTIFACT_DIR, 'publishing-purchase-mobile.png'), fullPage: true });
  await owner.goto(`${origin}/dashboard/listings`);
  await owner.locator('.directory-owner-row').filter({ hasText: 'Browser free resource' }).getByRole('link', { name: 'Choose publishing plan', exact: true }).click();
  await owner.getByRole('button', { name: 'Choose free plan', exact: true }).click();
  await expect(owner.getByText('Free plan', { exact: true })).toBeVisible();
  await expect(owner.getByRole('heading', { name: 'Active', exact: true })).toBeVisible();
  assert.deepEqual(errors, []);
  console.log('Paid browser: plan validation/edit, owner review, forged return, Stripe redirect, Paddle initialization/completion, cancellation confirmation, free plan, keyboard and mobile passed.');
} catch (error) {
  console.error(logs); throw error;
} finally {
  clearTimeout(watchdog); await browser?.close(); await stop();
}
