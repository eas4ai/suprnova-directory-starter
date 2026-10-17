import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { readFileSync, writeFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const cases = [
  ['SEO-001', 'src/seo/settings.rs', 'self.description = text(&self.description, "description", 320)?;', 'self.description = String::new();', /A saved site description/],
  ['SEO-002', 'frontend/src/components/PublicMetadata.vue', '    <meta head-key="twitter:card" name="twitter:card" :content="seo.image ? \'summary_large_image\' : \'summary\'" />', '', /initial HTML omitted name="twitter:card"/],
  ['SEO-003', 'src/seo/discovery.rs', "r.id = listings.approved_revision_id AND COALESCE(json_extract(r.seo, '$.noindex'), 0) = 0", 'r.id = listings.approved_revision_id', /SEO-003: noindex listing entered sitemap/],
  ['SEO-004', 'src/seo/redirects.rs', 'Ok(Some(current))', 'Ok(Some("/articles".into()))', /SEO-004: redirect destination changed/],
  ['SEO-005', 'src/seo/report.rs', 'let mut messages = Vec::new();', 'let mut preview = preview; preview.title.push_str(" broken-preview"); let mut messages = Vec::new();', /SEO-005: preview disagrees with public metadata/],
  ['SEO-006', 'src/seo/markdown.rs', '.header("Cache-Control", "no-store")\n            .header("X-Content-Type-Options", "nosniff")', '.header("Cache-Control", "public, max-age=3600")\n            .header("X-Content-Type-Options", "nosniff")', /SEO-006: Markdown must disable shared caching/],
];
function run(working, env, command, args) {
  const result = spawnSync(command, args, { cwd: working, env, encoding: 'utf8', timeout: 600000, detached: true, maxBuffer: 20 * 1024 * 1024 });
  if (result.pid) {
    try { process.kill(-result.pid, 'SIGKILL'); }
    catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
  if (result.error) throw result.error;
  return { status: result.status, output: result.stdout + result.stderr };
}
const working = snapshot(source, 'seo-negative-');
try {
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, { APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'https://catalog.example.test', APP_NAME: 'Directory',
    APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test', BILLING_CHECKOUT_MODE: 'test',
    DB_LOGGING: 'false', SESSION_SECURE: 'false', DIRECTORY_MEDIA_ROOT: join(working, 'storage/private/directory'), TMPDIR: working,
    CARGO_BUILD_JOBS: '2', CARGO_PROFILE_DEV_DEBUG: '0', CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: join(source, 'target') });
  function prerequisite(command, args) {
    const result = run(working, env, command, args);
    assert.equal(result.status, 0, `Prerequisite failed: ${command}\n${result.output.slice(-5000)}`);
  }
  prerequisite('bun', ['install', '--frozen-lockfile', '--cwd', 'frontend']);
  prerequisite('bun', ['run', '--cwd', 'frontend', 'build']);
  prerequisite('bun', ['run', '--cwd', 'frontend', 'build:ssr']);
  for (const [requirement, file, before, after, expected] of cases) {
    console.log(`${requirement}: testing violating copy`);
    const path = join(working, file);
    const original = readFileSync(path, 'utf8');
    assert.equal(original.split(before).length, 2, `${requirement}: mutation must match once`);
    writeFileSync(path, original.replace(before, after));
    env.DATABASE_URL = `sqlite://${join(working, `${requirement}.db`)}`;
    try {
      if (file.endsWith('.vue')) prerequisite('bun', ['run', '--cwd', 'frontend', 'build:ssr']);
      const result = run(working, env, 'node', ['scripts/with-ssr.mjs', 'cargo', 'test', '--locked', '--test', 'seo_workflows', '--', '--nocapture']);
      assert.equal(result.status, 1, `${requirement}: expected runtime test failure\n${result.output.slice(-6000)}`);
      assert.match(result.output, /test seo_workflows_contract \.\.\. FAILED/);
      assert.match(result.output, /Verification command exited 101/);
      assert.match(result.output, expected);
      const start = result.output.indexOf('panicked at tests/seo_workflows.rs');
      console.log(`${requirement} rejected: ${result.output.slice(start, start + 800)}`);
    } finally {
      writeFileSync(path, original);
      if (file.endsWith('.vue')) prerequisite('bun', ['run', '--cwd', 'frontend', 'build:ssr']);
    }
  }
} finally {
  rmSync(working, { recursive: true, force: true });
}
