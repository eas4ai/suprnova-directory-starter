import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run overview journeys with the real SSR renderer');
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
  await page.getByLabel('Email address', { exact: true }).fill(`overview-${name}@example.test`);
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(origin + '/dashboard');
  await page.goto(origin + path);
}

async function screenshot(page, name) {
  if (!env.OVERVIEW_ARTIFACT_DIR) return;
  await mkdir(env.OVERVIEW_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.OVERVIEW_ARTIFACT_DIR, name), fullPage: true, animations: 'disabled' });
}

async function overview(page) {
  for (const [metric, count] of [['published-listings', 2], ['pending-reviews', 2], ['published-articles', 1]]) {
    await expect(page.locator(`[data-metric="${metric}"] .overview-count`)).toHaveText(String(count));
  }
  await expect(page.locator('.overview-activity li')).toHaveCount(5);
  await expect(page.getByText('Your directory has no published listings yet.', { exact: true })).toHaveCount(0);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Overview overflows viewport');
}

async function contrast(page) {
  let samples;
  await expect.poll(async () => {
    samples = await page.locator('[data-metric] dt, [data-metric] dd, .navigation-group-label').evaluateAll(elements => {
      const context = document.createElement('canvas').getContext('2d');
      function luminance(color) {
        context.clearRect(0, 0, 1, 1); context.fillStyle = color; context.fillRect(0, 0, 1, 1);
        const rgb = [...context.getImageData(0, 0, 1, 1).data].slice(0, 3).map(value => {
          const c = value / 255; return c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4;
        });
        return .2126 * rgb[0] + .7152 * rgb[1] + .0722 * rgb[2];
      }
      return elements.filter(element => element.getClientRects().length).map(element => {
        const style = getComputedStyle(element);
        let background = style.backgroundColor;
        for (let parent = element.parentElement; background === 'rgba(0, 0, 0, 0)' && parent; parent = parent.parentElement) background = getComputedStyle(parent).backgroundColor;
        const [a, b] = [luminance(style.color), luminance(background)].sort((x, y) => y - x);
        return { text: element.textContent, ratio: (a + .05) / (b + .05) };
      });
    });
    return samples.length > 0 && samples.every(sample => sample.ratio >= 4.5);
  }, { message: 'Overview counts and navigation labels meet text contrast' }).toBe(true);
}

const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  await login(page, 'admin');
  await overview(page);
  const navigation = page.getByRole('navigation', { name: 'Administration', exact: true });
  await expect(navigation.locator('.navigation-group-label')).toHaveText(['Workspace', 'Content', 'Billing', 'Administration', 'Account']);
  await contrast(page);
  await screenshot(page, 'overview-desktop-light.png');

  await page.getByRole('link', { name: 'Review pending listings', exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(origin + '/admin/listings');
  await expect(page.locator('.directory-owner-list > li')).toHaveCount(2);
  await navigation.getByRole('link', { name: 'Overview', exact: true }).click();
  await overview(page);
  await page.getByRole('link', { name: 'View published articles', exact: true }).click();
  await expect(page).toHaveURL(origin + '/admin/articles?state=published');
  await expect(page.locator('.editorial-admin-list > li')).toHaveCount(1);
  await navigation.getByRole('link', { name: 'Overview', exact: true }).click();
  await page.getByRole('button', { name: 'Dark mode', exact: true }).focus();
  await page.keyboard.press('Space');
  await expect(page.locator('[data-shell="admin"]')).toHaveAttribute('data-theme', 'dark');
  await overview(page); await contrast(page);
  await screenshot(page, 'overview-desktop-dark.png');
  await page.setViewportSize({ width: 320, height: 844 });
  await overview(page); await contrast(page);
  await screenshot(page, 'overview-mobile-dark.png');
  const menu = page.getByRole('button', { name: 'Open navigation', exact: true });
  await menu.focus(); await page.keyboard.press('Enter');
  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole('group', { name: 'Content', exact: true })).toBeVisible();
  await screenshot(page, 'overview-mobile-navigation.png');
  await dialog.getByRole('link', { name: 'Articles', exact: true }).focus();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(origin + '/admin/articles');
  await expect(dialog).not.toBeVisible();

  for (const [name, visible] of [['moderator', 'published-listings'], ['editor', 'published-articles'], ['billing', null]]) {
    const delegated = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    const actor = await delegated.newPage();
    await login(actor, name);
    for (const metric of ['published-listings', 'published-articles']) {
      await expect(actor.locator(`[data-metric="${metric}"]`)).toHaveCount(metric === visible ? 1 : 0);
    }
    await expect(actor.locator('.overview-activity')).toHaveCount(0);
    const nav = actor.getByRole('navigation', { name: 'Administration', exact: true });
    for (const [label, owner] of [['Listing reviews', 'moderator'], ['Articles', 'editor'], ['Payment providers', 'billing']]) {
      await expect(nav.getByRole('link', { name: label, exact: true })).toHaveCount(name === owner ? 1 : 0);
    }
    await screenshot(actor, `overview-${name}.png`);
    await delegated.close();
  }
  // Use the actual owner and editorial workflows to make the public counts zero.
  const ownerContext = await browser.newContext();
  const owner = await ownerContext.newPage();
  await login(owner, 'owner', '/dashboard/listings');
  for (let remaining = 4; remaining > 0; remaining--) {
    const editable = owner.getByRole('link', { name: 'Edit listing', exact: true });
    await expect(editable).toHaveCount(remaining);
    await editable.first().click();
    owner.once('dialog', async dialog => {
      assert.match(dialog.message(), /^Archive this listing\?/);
      await dialog.accept();
    });
    await owner.getByRole('button', { name: 'Archive listing', exact: true }).click();
    await expect(owner).toHaveURL(origin + '/dashboard/listings');
  }
  await ownerContext.close();
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(origin + '/admin/articles?state=published');
  await page.locator('.editorial-admin-list > li').getByRole('link').click();
  page.once('dialog', async dialog => {
    assert.match(dialog.message(), /^Unpublish this article\?/);
    await dialog.accept();
  });
  await page.getByRole('button', { name: 'Unpublish article', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Unpublish article', exact: true })).toHaveCount(0);
  await page.goto(origin + '/admin');
  for (const metric of ['published-listings', 'pending-reviews', 'published-articles']) {
    await expect(page.locator(`[data-metric="${metric}"] .overview-count`)).toHaveText('0');
  }
  await expect(page.getByText('No listings are public right now.', { exact: true })).toBeVisible();
  await expect(page.getByText('No articles are public right now.', { exact: true })).toBeVisible();
  await expect(page.getByText('No listings are waiting for review.', { exact: true })).toBeVisible();
  await screenshot(page, 'overview-empty.png');
  expect(errors).toEqual([]);
  console.log('Overview counts, queue links, delegated data/navigation, keyboard/mobile themes, text contrast and real empty states passed.');
} catch (error) {
  if (browser) for (const context of browser.contexts()) for (const page of context.pages()) {
    try { await screenshot(page, 'overview-failure.png'); }
    catch (captureError) { console.error('Failure screenshot unavailable:', captureError.message); }
  }
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
