import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Use the real SSR worker');
assert.ok(process.env.CARGO_TARGET_DIR, 'Use a disposable verification environment');
const probe = createServer();
await new Promise(resolve => probe.listen(0, '127.0.0.1', resolve));
const port = probe.address().port;
await new Promise(resolve => probe.close(resolve));
const origin = `http://127.0.0.1:${port}`;
const env = { ...process.env, APP_ENV: 'production', APP_URL: origin,
  SERVER_HOST: '127.0.0.1', SERVER_PORT: String(port), SITE_THEME: '', SITE_ACCENT: '#146b56',
  MAIL_DRIVER: 'log', MAIL_ALLOW_NON_DELIVERING_IN_PRODUCTION: 'true', RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION: 'true' };
const presets = {
  zinc: ['#3f3f46', '#a1a1aa'], blue: ['#1d4ed8', '#60a5fa'],
  indigo: ['#4338ca', '#818cf8'], violet: ['#6d28d9', '#a78bfa'],
  emerald: ['#047857', '#34d399'], teal: ['#0f766e', '#2dd4bf'],
  rose: ['#be123c', '#fb7185'], orange: ['#c2410c', '#fb923c'],
};
let server, browser;
let logs = '';
const errors = [];
async function stop() {
  if (!server || server.exitCode !== null || server.signalCode !== null) return;
  server.kill('SIGTERM');
  await Promise.race([new Promise(resolve => server.once('exit', resolve)), delay(6000)]);
  if (server.exitCode === null && server.signalCode === null) {
    server.kill('SIGKILL');
    await new Promise(resolve => server.once('exit', resolve));
  }
}
async function start(preset = '', accent = '#146b56') {
  await stop();
  server = spawn(join(env.CARGO_TARGET_DIR, 'debug/directory'), ['serve', '--no-migrate'], {
    env: { ...env, SITE_THEME: preset, SITE_ACCENT: accent }, stdio: ['ignore', 'pipe', 'pipe'],
  });
  for (const stream of [server.stdout, server.stderr]) stream.on('data', data => { logs = (logs + data).slice(-12000); });
  let lastError;
  for (let attempt = 0; attempt < 150; attempt++) {
    if (server.exitCode !== null) throw new Error(`Application exited: ${logs}`);
    try { if ((await fetch(`${origin}/_suprnova/health`, { signal: AbortSignal.timeout(500) })).ok) return; }
    catch (error) { lastError = error; }
    await delay(100);
  }
  throw new Error(`Application did not become ready: ${logs}`, { cause: lastError });
}
async function appearance(page, mode, accent) {
  await expect.poll(() => page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(mode);
  await expect(page.getByRole('button', { name: 'Dark mode', exact: true })).toHaveAttribute('aria-pressed', String(mode === 'dark'));
  if (accent) assert.equal(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--site-accent').trim()), accent);
  assert.equal(await page.evaluate(() => getComputedStyle(document.body).backgroundColor), mode === 'dark' ? 'rgb(9, 9, 11)' : 'rgb(255, 255, 255)');
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Page overflows horizontally');
}
async function contrast(page, selector) {
  // Read final colors, not a frame halfway through the button's hover transition.
  await page.locator(selector).evaluateAll(elements => Promise.all(elements.flatMap(element => {
    getComputedStyle(element).backgroundColor;
    return element.getAnimations().map(animation => animation.finished);
  })));
  let results;
  await expect.poll(async () => {
    results = await page.locator(selector).evaluateAll(elements => {
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = 1;
      const context = canvas.getContext('2d', { willReadFrequently: true });
      function luminance(color) {
        context.clearRect(0, 0, 1, 1);
        context.fillStyle = color;
        context.fillRect(0, 0, 1, 1);
        const rgb = [...context.getImageData(0, 0, 1, 1).data].slice(0, 3).map(value => {
          const c = value / 255;
          return c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4;
        });
        return .2126 * rgb[0] + .7152 * rgb[1] + .0722 * rgb[2];
      }
      return elements.filter(element => element.getClientRects().length).map(element => {
        const style = getComputedStyle(element);
        let background = style.backgroundColor;
        for (let parent = element.parentElement; background === 'rgba(0, 0, 0, 0)' && parent; parent = parent.parentElement) background = getComputedStyle(parent).backgroundColor;
        const [a, b] = [luminance(style.color), luminance(background)].sort((x, y) => y - x);
        return { text: element.textContent.trim().slice(0, 60), ratio: (a + .05) / (b + .05) };
      });
    });
    return results.length > 0 && results.every(result => result.ratio >= 4.5);
  }, { message: `Settled colors must meet contrast for ${selector}` }).toBe(true);
  assert.ok(results.length, `No visible elements matched ${selector}`);
  for (const result of results) assert.ok(result.ratio >= 4.5, `${selector} ${JSON.stringify(result)}`);
}
async function screenshot(page, name) {
  if (!env.APPEARANCE_ARTIFACT_DIR) return;
  await mkdir(env.APPEARANCE_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.APPEARANCE_ARTIFACT_DIR, name), fullPage: true, animations: 'disabled' });
}
async function login(page) {
  await page.goto(origin + '/login');
  await page.getByLabel('Email address', { exact: true }).fill('adoption-browser-admin@example.test');
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(origin + '/dashboard');
}

const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 180000);
try {
  await start();
  browser = await chromium.launch();
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const noJsContext = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  const noJs = await noJsContext.newPage();
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => { if (/hydration/i.test(message.text())) errors.push(message.text()); });
  await page.goto(origin);
  await appearance(page, 'light', '#146b56');
  await screenshot(page, 'default-light.png');
  await contrast(page, '.button-primary');
  const toggle = page.getByRole('button', { name: 'Dark mode', exact: true });
  await toggle.focus();
  await page.keyboard.press('Space');
  await appearance(page, 'dark');
  await contrast(page, '.button-primary');
  await page.getByRole('link', { name: 'Explore', exact: true }).click();
  await expect(page).toHaveURL(origin + '/listings');
  await appearance(page, 'dark');
  await page.reload();
  await appearance(page, 'dark');
  const cookie = (await context.cookies()).find(value => value.name === 'site_appearance');
  assert.equal(cookie.value, 'dark');
  assert.equal(cookie.path, '/');
  assert.equal(cookie.sameSite, 'Lax');
  assert.ok(cookie.expires > Date.now() / 1000 + 3600 * 24 * 300);

  // Separate requests must retain their own appearance even in a shared worker.
  await noJsContext.addCookies([{ name: 'site_appearance', value: 'dark', url: origin }]);
  await noJs.goto(origin);
  await appearance(noJs, 'dark');
  await screenshot(noJs, 'default-dark-mobile-no-js.png');
  for (let iteration = 0; iteration < 3; iteration++) {
    const [dark, light] = await Promise.all([
      fetch(origin, { headers: { cookie: 'site_appearance=dark' } }).then(response => response.text()),
      fetch(origin).then(response => response.text()),
    ]);
    assert.match(dark, /color-scheme: dark/);
    assert.match(light, /color-scheme: light/);
  }
  await noJsContext.addCookies([{ name: 'site_appearance', value: 'invalid', url: origin }]);
  await noJs.reload();
  await appearance(noJs, 'light', '#146b56');

  await page.goto(origin + '/listings/demo-approved');
  await appearance(page, 'dark');
  await expect(page.locator('.directory-description')).toBeVisible();
  await page.locator('.directory-description').evaluate(element => {
    // Exercise nested Markdown colors without changing stored data.
    element.insertAdjacentHTML('beforeend', '<h2>Example heading</h2><p><strong>Bold text</strong> and <a href="#">a resource link</a>.</p><pre><code>example code</code></pre>');
  });
  await contrast(page, '.directory-description h2, .directory-description strong, .directory-description a, .directory-description code');
  await screenshot(page, 'listing-dark-typography.png');
  await page.goto(origin + '/articles');
  await appearance(page, 'dark');
  await contrast(page, '.editorial-articles h2, .editorial-articles p');
  await page.goto(origin + '/login');
  await appearance(page, 'dark');
  await contrast(page, '.form-input, .button-primary');
  await screenshot(page, 'sign-in-dark.png');
  await login(page);
  await page.goto(origin + '/admin');
  await appearance(page, 'dark');
  await contrast(page, '.button-primary');
  await screenshot(page, 'admin-dark.png');
  await page.setViewportSize({ width: 320, height: 844 });
  await appearance(page, 'dark');
  await toggle.focus();
  await page.keyboard.press('Enter');
  await appearance(page, 'light', '#146b56');
  await toggle.click();
  await appearance(page, 'dark');
  await page.getByRole('button', { name: 'Open navigation', exact: true }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await contrast(page, '.app-dialog');
  await page.keyboard.press('Escape');
  await toggle.click();
  await page.setViewportSize({ width: 1440, height: 1000 });

  for (const [preset, [light, dark]] of Object.entries(presets)) {
    console.log(`Checking ${preset} in public/admin light and dark modes`);
    await start(preset, '#673ab7');
    await context.addCookies([{ name: 'site_appearance', value: 'light', url: origin }]);
    for (const path of ['/', '/admin']) {
      await page.goto(origin + path);
      await appearance(page, 'light', light);
      await contrast(page, '.button-primary');
      await toggle.click();
      await appearance(page, 'dark', dark);
      await contrast(page, '.button-primary');
      await page.locator('.button-primary:visible').first().hover();
      await contrast(page, '.button-primary');
      await toggle.click();
    }
    await noJsContext.addCookies([{ name: 'site_appearance', value: 'dark', url: origin }]);
    await noJs.goto(origin);
    await appearance(noJs, 'dark', dark);
    await contrast(noJs, '.button-primary');
    await screenshot(noJs, `${preset}-dark-mobile.png`);
  }
  await start('', '#673ab7');
  await page.goto(origin);
  await appearance(page, 'light', '#673ab7');
  await toggle.click();
  await contrast(page, '.button-primary');
  assert.deepEqual(errors, []);
  console.log('Blank/custom accent, eight presets, light/dark contrast, keyboard switch, reload/navigation, mobile shells and isolated no-JavaScript SSR passed.');
} catch (error) {
  if (browser) for (const context of browser.contexts()) for (const page of context.pages()) {
    try { await screenshot(page, `failure-${context === browser.contexts()[0] ? 'browser' : 'no-js'}.png`); }
    catch (captureError) { console.error('Could not capture failure:', captureError.message); }
  }
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
