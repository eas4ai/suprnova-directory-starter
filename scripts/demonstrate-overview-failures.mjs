import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { readFileSync, writeFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const cases = [
  ['OVR-001', 'src/controllers/admin.rs', '.filter(listings::queries::eligible(now))', '.filter(listing::Column::Archived.eq(false))'],
  ['OVR-002', 'src/controllers/admin.rs', '.has_permission_to(listings::MODERATE_PERMISSION)', '.has_permission_to(crate::billing::ADMIN_PERMISSION)'],
  ['OVR-003', 'frontend/src/pages/admin/Overview.vue', 'href="/admin/listings"', 'href="/admin/articles"'],
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

for (const [requirement, file, before, after] of cases) {
  const working = snapshot(source, 'overview-negative-');
  try {
    const path = join(working, file);
    const original = readFileSync(path, 'utf8');
    assert.equal(original.split(before).length, 2, `${requirement}: mutation must match exactly once`);
    let mutated = original.replace(before, after);
    if (requirement === 'OVR-002') {
      const guard = '        articles::require_permission(user.id, listings::MODERATE_PERMISSION).await?;\n';
      assert.equal(mutated.split(guard).length, 2, 'OVR-002: secondary permission guard must match once');
      mutated = mutated.replace(guard, '');
    }
    writeFileSync(path, mutated);
    const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
    Object.assign(env, { APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'https://catalog.example.test', APP_NAME: 'Directory',
      APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test', BILLING_CHECKOUT_MODE: 'test',
      DATABASE_URL: `sqlite://${join(working, 'overview.db')}`, DB_LOGGING: 'false', SESSION_SECURE: 'false',
      DIRECTORY_MEDIA_ROOT: join(working, 'storage/private/directory'), TMPDIR: working,
      CARGO_BUILD_JOBS: '2', CARGO_PROFILE_DEV_DEBUG: '0', CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: join(source, 'target') });
    if (requirement === 'OVR-003') {
      for (const [command, args] of [
        ['cargo', ['build', '--locked', '--bins']],
        ['bun', ['install', '--frozen-lockfile', '--cwd', 'frontend']],
        ['bun', ['run', '--cwd', 'frontend', 'build']],
        ['bun', ['run', '--cwd', 'frontend', 'build:ssr']],
        ['cargo', ['test', '--locked', '--test', 'overview_workflows', '--', '--nocapture']],
      ]) {
        const result = run(working, env, command, args);
        assert.equal(result.status, 0, `${requirement}: prerequisite failed\n${result.output.slice(-4000)}`);
      }
      const result = run(working, env, 'node', ['scripts/with-ssr.mjs', 'node', 'frontend/tests/overview-workflows.mjs']);
      assert.equal(result.status, 1, `${requirement}: expected browser assertion failure`);
      assert.match(result.output, /toHaveURL/);
      assert.match(result.output, /\/admin\/listings/);
      assert.match(result.output, /\/admin\/articles/);
      console.log(`${requirement}: wrong queue destination rejected by the browser URL assertion.`);
    } else {
      const result = run(working, env, 'cargo', ['test', '--locked', '--test', 'overview_workflows', '--', '--nocapture']);
      assert.equal(result.status, 101, `${requirement}: expected a runtime test failure\n${result.output.slice(-4000)}`);
      assert.match(result.output, /test overview_workflows_contract \.\.\. FAILED/);
      assert.match(result.output, /panicked at tests\/overview_workflows.rs/);
      assert.match(result.output, requirement === 'OVR-001' ? /"published": Number\(4\)/ : /editor received listing_summary/);
      const start = result.output.indexOf('panicked at tests/overview_workflows.rs');
      console.log(`${requirement} rejected: ${result.output.slice(start, start + 500)}`);
    }
  } finally {
    rmSync(working, { recursive: true, force: true });
  }
}
