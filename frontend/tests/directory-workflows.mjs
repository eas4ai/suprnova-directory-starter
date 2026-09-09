import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir, readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

const probe = createServer();
await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin, SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port),
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
let server;
let browser;
let logs = '';
async function stop() {
  if (!server || server.exitCode !== null || server.signalCode !== null) return;
  server.kill('SIGTERM');
  await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(6000)]);
  if (server.exitCode === null && server.signalCode === null) server.kill('SIGKILL');
}
async function start() {
  let lastProbeError;
  server = spawn(join(env.CARGO_TARGET_DIR, 'debug/directory'), ['serve', '--no-migrate'], { env, stdio: ['ignore', 'pipe', 'pipe'] });
  for (const stream of [server.stdout, server.stderr]) stream.on('data', data => { logs = (logs + data).slice(-20000); });
  for (let attempt = 0; attempt < 150; attempt++) {
    if (server.exitCode !== null) throw new Error(`Application exited: ${logs}`);
      try { if ((await fetch(`${origin}/_suprnova/health`, { signal: AbortSignal.timeout(500) })).ok) return; }
      catch (error) { lastProbeError = error; }
    await delay(100);
  }
    throw new Error(`Application did not start: ${logs}`, { cause: lastProbeError });
}
const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const ownerContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const adminContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const publicContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const owner = await ownerContext.newPage();
  const admin = await adminContext.newPage();
  const visitor = await publicContext.newPage();
  const errors = [];
  for (const page of [owner, admin, visitor]) {
    page.on('pageerror', error => errors.push(error.message));
    page.on('dialog', dialog => { errors.push(`Unexpected browser dialog: ${dialog.message()}`); void dialog.dismiss(); });
  }
  async function login(page, email) {
    await page.goto(`${origin}/login`);
    await page.getByLabel('Email address', { exact: true }).fill(email);
    await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page).toHaveURL(`${origin}/dashboard`);
  }
  await login(owner, 'directory-owner@example.test');
  await owner.getByRole('link', { name: 'Your listings', exact: true }).first().click();
  await owner.getByRole('link', { name: 'Create listing', exact: true }).click();
  await owner.getByLabel('Title', { exact: true }).fill('Browser resource');
  await owner.getByLabel('Summary', { exact: true }).fill('A browser-created listing for the complete owner journey.');
  await owner.getByLabel('Description', { exact: true }).fill('**Useful resource**\n\n<script>window.directoryInjected = true</script>\n[unsafe](javascript:alert(1))');
  await owner.getByLabel('Website URL', { exact: true }).fill('https://example.test/browser');
  await owner.getByRole('checkbox', { name: 'Software', exact: true }).check();
  const file = (await readdir(env.DIRECTORY_MEDIA_ROOT)).find(file => file.endsWith('.png'));
  assert.ok(file, 'The synthetic image fixture is missing');
  await owner.getByLabel('Listing image (optional)', { exact: true }).setInputFiles({ name: 'resource.png', mimeType: 'image/png', buffer: await readFile(join(env.DIRECTORY_MEDIA_ROOT, file)) });
  await owner.getByLabel('Image alternative text', { exact: true }).fill('Small resource logo');
  const save = owner.getByRole('button', { name: 'Save draft', exact: true });
  await save.focus();
  await owner.keyboard.press('Enter');
  await expect(owner.getByRole('status')).toContainText('Draft saved.');
  const editUrl = owner.url();
  const id = editUrl.match(/\/listings\/(\d+)\/edit$/)?.[1];
  assert.ok(id, 'Create must redirect to the saved listing');
  await stop(); await start();
  await owner.reload();
  await expect(owner.getByLabel('Title', { exact: true })).toHaveValue('Browser resource');
  await expect(owner.getByAltText('Small resource logo', { exact: true })).toBeVisible();
  await owner.getByRole('button', { name: 'Submit for review', exact: true }).click();
  await expect(owner.getByText('This revision is awaiting review.', { exact: false })).toBeVisible();
  await login(admin, 'directory-moderator@example.test');
  await admin.goto(`${origin}/admin/listings`);
  await admin.getByRole('link', { name: 'Browser resource', exact: true }).click();
  await admin.getByRole('radio', { name: 'Reject revision', exact: true }).check();
  await admin.getByLabel('Reason for rejection', { exact: true }).fill('Please clarify the resource title.');
  await admin.getByRole('button', { name: 'Record decision', exact: true }).click();
  await expect(admin.getByRole('heading', { name: 'Current revision · rejected', exact: true })).toBeVisible();
  await owner.reload();
  await expect(owner.getByText('Please clarify the resource title.', { exact: false })).toBeVisible();
  await owner.getByLabel('Title', { exact: true }).fill('Browser resource clarified');
  await save.click();
  await expect(owner.getByRole('status')).toContainText('Draft saved.');
  await owner.getByRole('button', { name: 'Submit for review', exact: true }).click();
  await admin.reload();
  await admin.getByRole('radio', { name: 'Approve revision', exact: true }).check();
  await admin.getByRole('button', { name: 'Record decision', exact: true }).click();
  await expect(admin.getByRole('heading', { name: 'Current revision · approved', exact: true })).toBeVisible();
  await owner.goto(`${origin}/dashboard/listings`);
  const ownerRow = owner.locator('.directory-owner-row').filter({ has: owner.getByRole('link', { name: 'Browser resource clarified', exact: true }) });
  await expect(ownerRow.getByText('Awaiting payment', { exact: true })).toBeVisible();
  await expect(ownerRow.getByRole('link', { name: 'Choose publishing plan', exact: true })).toHaveAttribute('href', `/dashboard/listings/${id}/plans`);
  await visitor.goto(`${origin}/listings?q=Browser`);
  await expect(visitor.getByRole('heading', { name: 'No matching listings', exact: true })).toBeVisible();

  // A stale edit keeps the user's content and explains how to recover.
  await owner.goto(editUrl);
  const second = await ownerContext.newPage();
  await second.goto(editUrl);
  await second.getByLabel('Summary', { exact: true }).fill('The newer saved summary.');
  await second.getByRole('button', { name: 'Save draft', exact: true }).click();
  await expect(second.getByRole('status')).toContainText('Draft saved.');
  await owner.getByLabel('Summary', { exact: true }).fill('A stale summary to preserve.');
  await save.click();
  await expect(owner.getByRole('alert').filter({ hasText: 'Changes were not saved' })).toContainText('changed after you opened');
  await expect(owner.getByRole('alert').filter({ hasText: 'Changes were not saved' })).toBeFocused();
  await expect(owner.getByLabel('Summary', { exact: true })).toHaveValue('A stale summary to preserve.');
  await second.close();

  await visitor.goto(origin);
  await expect(visitor.getByRole('heading', { name: 'A place for good discoveries.', exact: true })).toBeVisible();
  await expect(visitor.getByRole('link', { name: 'Pagination Alpha', exact: true })).toBeVisible();
  await visitor.getByLabel('Search listings', { exact: true }).fill('Pagination');
  await visitor.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(visitor.locator('.directory-card')).toHaveCount(2);
  await visitor.getByRole('button', { name: 'Compact', exact: true }).focus();
  await visitor.keyboard.press('Enter');
  await expect(visitor.getByRole('button', { name: 'Compact', exact: true })).toHaveAttribute('aria-pressed', 'true');
  await visitor.getByRole('link', { name: 'Pagination Alpha', exact: true }).click();
  await expect(visitor.getByRole('heading', { name: 'Pagination Alpha', exact: true })).toBeVisible();
  assert.equal(await visitor.evaluate(() => Boolean(window.directoryInjected)), false);
  assert.equal(await visitor.locator('a[href^="javascript:"]').count(), 0);
  if (env.DIRECTORY_ARTIFACT_DIR) {
    await mkdir(env.DIRECTORY_ARTIFACT_DIR, { recursive: true });
    await admin.screenshot({ path: join(env.DIRECTORY_ARTIFACT_DIR, 'moderation-desktop.png'), fullPage: true });
  }
  await visitor.setViewportSize({ width: 390, height: 844 });
  await visitor.goto(`${origin}/listings`);
  const category = visitor.getByLabel('Category', { exact: true });
  await expect(category.locator('option[value="design"]')).toHaveCount(0);
  await category.selectOption('software');
  await visitor.getByLabel('Search listings', { exact: true }).fill('Pagination');
  await visitor.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(visitor.locator('.directory-card')).toHaveCount(2);
  await visitor.getByLabel('Search listings', { exact: true }).fill('No public listing matches this fixture');
  await visitor.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(visitor.getByRole('heading', { name: 'No matching listings', exact: true })).toBeVisible();
  assert.ok(await visitor.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), 'Directory results overflow on mobile');
  if (env.DIRECTORY_ARTIFACT_DIR) await visitor.screenshot({ path: join(env.DIRECTORY_ARTIFACT_DIR, 'directory-mobile.png'), fullPage: true });
  await owner.setViewportSize({ width: 390, height: 844 });
  assert.ok(await owner.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), 'Listing form overflows on mobile');
  expect(errors).toEqual([]);
  console.log('Owner save/restart/upload, rejection/resubmission/approval, unpaid visibility, stale edit recovery, public search/detail, keyboard and mobile journeys passed.');
} catch (error) {
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
