import assert from 'node:assert/strict';
import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { mkdir, readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

assert.ok(process.env.SSR_URL, 'Run this journey through scripts/with-ssr.mjs with the real renderer');
assert.ok(process.env.CARGO_TARGET_DIR, 'CARGO_TARGET_DIR is required');
assert.ok(process.env.DIRECTORY_MEDIA_ROOT, 'DIRECTORY_MEDIA_ROOT is required');
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
const suffix = String(Date.now());
const title = `Browser editorial ${suffix} </title><script>window.editorialInjected=true</script>`;
const slug = `browser-editorial-${suffix}`;
const nextSlug = `${slug}-revised`;
const categoryName = `Browser category ${suffix}`;
const categorySlug = `browser-category-${suffix}`;
const tagName = `Browser tag ${suffix}`;
const tagSlug = `browser-tag-${suffix}`;
const summary = 'A private draft becomes a published guide through explicit editorial actions.';
const body = '**Original browser article body**\n\n<script>window.editorialInjected = true</script>\n\n[unsafe](javascript:alert(1))';
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
  page.on('dialog', async dialog => {
    const expected = page.__expectedDialog;
    page.__expectedDialog = undefined;
    if (expected && dialog.message().includes(expected)) await dialog.accept();
    else { errors.push(`Unexpected dialog: ${dialog.message()}`); await dialog.dismiss(); }
  });
}
async function login(page, email) {
  await page.goto(`${origin}/login`);
  await page.getByLabel('Email address', { exact: true }).fill(email);
  await page.getByLabel('Password', { exact: true }).fill('fixture-password-123');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page).toHaveURL(`${origin}/dashboard`);
}
async function screenshot(page, name) {
  if (!env.EDITORIAL_ARTIFACT_DIR) return;
  await mkdir(env.EDITORIAL_ARTIFACT_DIR, { recursive: true });
  await page.screenshot({ path: join(env.EDITORIAL_ARTIFACT_DIR, name), fullPage: true });
}
async function saved(page) {
  await expect(page.getByRole('status').filter({ hasText: 'Changes saved.' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Save draft', exact: true })).toBeEnabled();
}
async function publicMissing(path) {
  const response = await fetch(origin + path, { redirect: 'follow' });
  assert.equal(response.status, 404, `Private article leaked at ${path}`);
}
async function sitemapContent() {
  const response = await fetch(origin + '/sitemap.xml');
  assert.equal(response.status, 200);
  const xml = await response.text();
  if (!xml.includes('<sitemapindex')) return xml;
  let combined = xml;
  for (const match of xml.matchAll(/<loc>([^<]+)<\/loc>/g)) {
    const url = new URL(match[1].replaceAll('&amp;', '&'));
    assert.equal(url.origin, origin, 'Sitemap uses configured application origin');
    const part = await fetch(url);
    assert.equal(part.status, 200);
    combined += await part.text();
  }
  return combined;
}
const watchdog = setTimeout(() => { server?.kill('SIGKILL'); void browser?.close(); process.exitCode = 1; }, 240000);
try {
  await start();
  browser = await chromium.launch();
  const adminContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const publicContext = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const noJsContext = await browser.newContext({ javaScriptEnabled: false });
  const memberContext = await browser.newContext();
  const admin = await adminContext.newPage();
  const visitor = await publicContext.newPage();
  const noJs = await noJsContext.newPage();
  const member = await memberContext.newPage();
  for (const page of [admin, visitor, noJs, member]) watchPage(page);
  await login(admin, 'editorial-admin@example.test');
  await login(member, 'editorial-member@example.test');
  const denied = await memberContext.request.get(origin + '/admin/articles', { maxRedirects: 0 });
  assert.equal(denied.status(), 302, 'Non-editor must be denied by the administration gate');
  assert.equal(denied.headers().location, '/dashboard');
  await member.goto(origin + '/admin/articles');
  await expect(member).toHaveURL(origin + '/dashboard');
  await expect(member.getByRole('link', { name: 'Create article', exact: true })).toHaveCount(0);

  // Create separate article taxonomy through the same controls administrators use.
  for (const [kind, name, termSlug] of [['category', categoryName, categorySlug], ['tag', tagName, tagSlug]]) {
    await admin.goto(`${origin}/admin/taxonomy?kind=${kind}`);
    await admin.getByLabel('Name', { exact: true }).fill(name);
    await admin.getByLabel('URL slug', { exact: true }).fill(termSlug);
    await admin.getByRole('button', { name: 'Create term', exact: true }).last().click();
    await expect(admin.getByRole('status')).toContainText('Taxonomy updated.');
    await expect(admin.getByRole('button', { name: new RegExp(name) })).toBeVisible();
  }
  await admin.goto(origin + '/admin/articles');
  await admin.getByRole('link', { name: 'Create article', exact: true }).click();
  await admin.getByLabel('Title', { exact: true }).fill(title);
  await admin.getByLabel('Summary', { exact: true }).fill(summary);
  await admin.getByLabel('Article body', { exact: true }).fill(body);
  await admin.getByLabel('URL slug', { exact: true }).fill(slug);
  await admin.getByRole('checkbox', { name: categoryName, exact: true }).check();
  await admin.getByRole('checkbox', { name: tagName, exact: true }).check();
  const file = (await readdir(env.DIRECTORY_MEDIA_ROOT)).find(name => name.endsWith('.png'));
  assert.ok(file, 'Synthetic PNG fixture is missing');
  await admin.getByLabel('Cover image (optional)', { exact: true }).setInputFiles({ name: 'editorial.png', mimeType: 'image/png', buffer: await readFile(join(env.DIRECTORY_MEDIA_ROOT, file)) });
  await admin.getByLabel('Image alternative text', { exact: true }).fill('Synthetic editorial cover');
  const save = admin.getByRole('button', { name: 'Save draft', exact: true });
  await save.focus();
  await admin.keyboard.press('Enter');
  await saved(admin);
  const editUrl = admin.url();
  const id = editUrl.match(/\/admin\/articles\/(\d+)\/edit$/)?.[1];
  assert.ok(id, 'Saving an article must redirect to its editor');
  await publicMissing('/articles/' + slug);
  const previewPromise = adminContext.waitForEvent('page');
  await admin.getByRole('link', { name: 'Preview saved draft (new tab)', exact: true }).click();
  const preview = await previewPromise;
  watchPage(preview);
  await expect(preview.getByRole('heading', { name: title, exact: true })).toBeVisible();
  await expect(preview.locator('meta[name="robots"]')).toHaveAttribute('content', /noindex/);
  await expect(preview.getByAltText('Synthetic editorial cover', { exact: true })).toBeVisible();
  assert.equal(await preview.evaluate(() => Boolean(window.editorialInjected)), false);
  assert.equal(await preview.locator('a[href^="javascript:"]').count(), 0);
  await screenshot(preview, 'article-private-preview.png');
  await preview.close();

  await admin.getByRole('button', { name: 'Publish article', exact: true }).click();
  await saved(admin);
  await expect(admin.getByRole('button', { name: 'Unpublish article', exact: true })).toBeVisible();
  await visitor.goto(origin + '/articles?q=' + encodeURIComponent(title));
  await expect(visitor.getByRole('link', { name: title, exact: true })).toBeVisible();
  await visitor.getByLabel('Category', { exact: true }).selectOption(categorySlug);
  await visitor.getByLabel('Tag', { exact: true }).selectOption(tagSlug);
  await visitor.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(visitor.getByRole('link', { name: title, exact: true })).toBeVisible();
  await visitor.getByRole('link', { name: title, exact: true }).click();
  await expect(visitor.getByRole('heading', { name: title, exact: true })).toBeVisible();
  await expect(visitor.locator('.editorial-prose')).toContainText('Original browser article body');
  assert.equal(await visitor.evaluate(() => Boolean(window.editorialInjected)), false);
  assert.equal(await visitor.locator('a[href^="javascript:"]').count(), 0);
  const initialHtml = await fetch(origin + '/articles/' + slug).then(response => response.text());
  assert.match(initialHtml, /Original browser article body/, 'Article content must be in initial HTML');
  assert.match(initialHtml, /property="og:title"/, 'Open Graph metadata must be in initial HTML');
  await noJs.goto(origin + '/articles/' + slug);
  await expect(noJs.getByRole('heading', { name: title, exact: true })).toBeVisible();
  await expect(noJs.locator('.editorial-prose')).toContainText('Original browser article body');
  await expect(noJs.locator('link[rel="canonical"]')).toHaveAttribute('href', origin + '/articles/' + slug);
  await expect(noJs.locator('meta[property="og:title"]')).toHaveAttribute('content', `${title} | ${env.APP_NAME ?? 'Directory'}`);
  await expect(noJs.locator('meta[name="description"]')).toHaveAttribute('content', summary);
  await expect(noJs).toHaveTitle(`${title} | ${env.APP_NAME ?? 'Directory'}`);
  const structured = JSON.parse(await noJs.locator('script[type="application/ld+json"]').textContent());
  assert.equal(structured[0]['@type'], 'Article');
  assert.equal(structured[0].headline, title);
  assert.equal(structured[0].mainEntityOfPage, origin + '/articles/' + slug);
  assert.equal(structured[1]['@type'], 'BreadcrumbList');
  await screenshot(visitor, 'article-published-desktop.png');

  // Saving a revision is private until another explicit publication.
  await admin.getByLabel('Article body', { exact: true }).fill('**Revised browser article body**');
  await admin.getByLabel('URL slug', { exact: true }).fill(nextSlug);
  await expect(admin.getByRole('button', { name: 'Publish saved changes', exact: true })).toBeDisabled();
  await save.click();
  await saved(admin);
  await visitor.reload();
  await expect(visitor.locator('.editorial-prose')).toContainText('Original browser article body');
  await publicMissing('/articles/' + nextSlug);
  await admin.getByRole('button', { name: 'Publish saved changes', exact: true }).click();
  await saved(admin);
  const redirect = await fetch(origin + '/articles/' + slug, { redirect: 'manual' });
  assert.ok([301, 302, 307, 308].includes(redirect.status), 'Old published slug must redirect');
  assert.equal(new URL(redirect.headers.get('location'), origin).pathname, '/articles/' + nextSlug);
  await visitor.goto(origin + '/articles/' + nextSlug);
  await expect(visitor.locator('.editorial-prose')).toContainText('Revised browser article body');

  // A concurrent edit must reject stale writes and preserve the operator's text.
  const second = await adminContext.newPage();
  watchPage(second);
  await second.goto(editUrl);
  await second.getByLabel('Summary', { exact: true }).fill('A newer saved summary from the second editor.');
  await second.getByRole('button', { name: 'Save draft', exact: true }).click();
  await saved(second);
  await admin.getByLabel('Summary', { exact: true }).fill('Preserve this stale editorial summary.');
  await save.click();
  const alert = admin.getByRole('alert').filter({ hasText: 'Changes were not saved' });
  await expect(alert).toBeVisible();
  await expect(alert).toBeFocused();
  await expect(admin.getByLabel('Summary', { exact: true })).toHaveValue('Preserve this stale editorial summary.');
  await expect(alert.getByRole('link', { name: 'Reload latest revision', exact: true })).toBeVisible();
  await screenshot(admin, 'article-stale-edit.png');
  admin.__expectedDialog = 'Unsaved changes';
  await alert.getByRole('link', { name: 'Reload latest revision', exact: true }).click();
  await expect(admin.getByLabel('Summary', { exact: true })).toHaveValue('A newer saved summary from the second editor.');
  await second.close();

  // In-use taxonomy is protected; disabling preserves the article relationship.
  const taxonomy = await adminContext.newPage();
  watchPage(taxonomy);
  await taxonomy.goto(origin + '/admin/taxonomy?kind=category');
  await taxonomy.getByRole('button', { name: new RegExp(categoryName) }).click();
  await expect(taxonomy.getByLabel('URL slug', { exact: true })).toHaveAttribute('readonly', '');
  taxonomy.__expectedDialog = 'Remove this unused term?';
  await taxonomy.getByRole('button', { name: 'Remove term', exact: true }).click();
  await expect(taxonomy.getByRole('alert').filter({ hasText: 'Changes were not saved' })).toBeVisible();
  await taxonomy.getByRole('checkbox', { name: 'Active', exact: true }).uncheck();
  await taxonomy.getByRole('button', { name: 'Save term', exact: true }).click();
  await expect(taxonomy.getByRole('status')).toContainText('Taxonomy updated.');
  await visitor.reload();
  await expect(visitor.getByRole('heading', { name: title, exact: true })).toBeVisible();
  await taxonomy.setViewportSize({ width: 390, height: 844 });
  assert.ok(await taxonomy.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Taxonomy overflows on mobile');
  await screenshot(taxonomy, 'taxonomy-mobile.png');
  await taxonomy.close();

  await admin.setViewportSize({ width: 390, height: 844 });
  assert.ok(await admin.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Article editor overflows on mobile');
  await screenshot(admin, 'article-editor-mobile.png');
  await visitor.setViewportSize({ width: 390, height: 844 });
  assert.ok(await visitor.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Public article overflows on mobile');
  await screenshot(visitor, 'article-public-mobile.png');
  const rssBefore = await fetch(origin + '/feed.xml').then(response => response.text());
  assert.ok(rssBefore.includes('/articles/' + nextSlug), 'Published article missing from RSS');
  assert.ok((await sitemapContent()).includes('/articles/' + nextSlug), 'Published article missing from sitemap');
  admin.__expectedDialog = 'Unpublish this article?';
  await admin.getByRole('button', { name: 'Unpublish article', exact: true }).click();
  await saved(admin);
  await publicMissing('/articles/' + nextSlug);
  await publicMissing('/articles/' + slug);
  await visitor.goto(origin + '/articles?q=' + encodeURIComponent(title));
  await expect(visitor.getByRole('heading', { name: 'No articles match your search', exact: true })).toBeVisible();
  assert.ok(!(await fetch(origin + '/feed.xml').then(response => response.text())).includes('/articles/' + nextSlug), 'Unpublished article remains in RSS');
  assert.ok(!(await sitemapContent()).includes('/articles/' + nextSlug), 'Unpublished article remains in sitemap');
  await visitor.goto(origin + '/articles');
  const next = visitor.getByRole('link', { name: 'Next', exact: true });
  await expect(next).toBeVisible();
  await next.click();
  await expect(visitor).toHaveURL(/page=2/);
  assert.ok(await visitor.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Article results overflow on mobile');
  await screenshot(visitor, 'articles-mobile.png');
  expect(errors).toEqual([]);
  console.log('Editorial authoring/upload/preview/publication, initial HTML without JavaScript, safe Markdown, taxonomy protection, stale edits, slug redirects, feeds, pagination, keyboard and mobile journeys passed.');
} catch (error) {
  if (browser && env.EDITORIAL_ARTIFACT_DIR) {
    let index = 0;
    for (const context of browser.contexts()) {
      for (const page of context.pages()) {
        try { await screenshot(page, 'editorial-failure-' + index++ + '.png'); }
        catch (captureError) { console.error('Failure screenshot unavailable:', captureError.message); }
      }
    }
  }
  console.error(logs);
  throw error;
} finally {
  clearTimeout(watchdog);
  await browser?.close();
  await stop();
}
