import assert from 'node:assert/strict';
import { randomBytes } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { readFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[3] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const task = process.argv[2];
const requirements = { build: 'FWU-001', migration: 'FWU-002', auth: 'FWU-003', http: 'FWU-004', console: 'FWU-005', regression: 'FWU-006' };
const requirement = requirements[task];
const read = path => readFileSync(join(source, path), 'utf8');
let working;

function run(command, args, cwd = source, env = process.env, expected = 0) {
  console.log(`[framework-${task}] ${command} ${args.join(' ')}`);
  const result = spawnSync(command, args, { cwd, env, encoding: 'utf8', detached: true, timeout: 30 * 60 * 1000, maxBuffer: 32 * 1024 * 1024 });
  if (result.pid) {
    try { process.kill(-result.pid, 'SIGKILL'); }
    catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
  if (result.error) throw result.error;
  process.stdout.write(result.stdout ?? '');
  process.stderr.write(result.stderr ?? '');
  assert.equal(result.status, expected, `${command} exited ${result.status} (signal ${result.signal ?? 'none'})`);
  return `${result.stdout ?? ''}${result.stderr ?? ''}`;
}

function isolated() {
  working = snapshot(source, `directory-framework-${task}-`);
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, {
    APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'http://directory.test',
    APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test',
    DATABASE_URL: `sqlite://${join(working, `${task}.db`)}`, DB_LOGGING: 'false',
    SESSION_SECURE: 'false', TMPDIR: working,
    DIRECTORY_MEDIA_ROOT: join(working, 'storage/private/directory'),
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2', CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: join(source, 'target'),
  });
  return env;
}

function integration(suite) {
  const env = isolated();
  const output = run('cargo', ['test', '--locked', '--test', suite, '--', '--nocapture'], working, env);
  assert.match(output, /test result: ok\. [1-9][0-9]* passed; 0 failed/, 'The compatibility suite must execute tests');
}

try {
  assert.ok(requirement, 'Choose build, migration, auth, http, console or regression.');
  // These cheap prerequisite checks identify the old configuration before an
  // expensive run. They supplement, never replace, the behavioral checks below.
  if (task === 'build') {
    assert.match(read('Cargo.toml'), /suprnova = .*tag = "v4\.2\.0"/, 'FWU-001: framework must select v4.2.0');
    assert.match(read('Cargo.toml'), /suprnova-markdown = .*rev = "6f27ecae53babc0fd637c23b41bc5b244543c78e"/, 'FWU-001: use the published compatible Markdown revision');
    run('node', ['scripts/verify-foundation.mjs', 'build']);
  } else if (task === 'migration') {
    assert.match(read('src/migrations/mod.rs'), /RbacToSpatieLayout/, 'FWU-002: existing RBAC data needs the framework compatibility migration');
    assert.match(read('src/migrations/mod.rs'), /SessionUserKey::Integer/, 'FWU-002: preserve the existing signed account key');
    integration('framework_upgrade_migration');
    run('node', ['scripts/verify-foundation.mjs', 'database']);
  } else if (task === 'auth') {
    assert.match(read('src/accounts/provider.rs'), /rehash_password_if_required/, 'FWU-003: the account wrapper must forward credential rehash');
    integration('framework_upgrade_auth');
    run('node', ['scripts/verify-foundation.mjs', 'accounts']);
  } else if (task === 'http') {
    assert.match(read('frontend/vite.config.ts'), /['"]src\/app\.css['"]/, 'FWU-004: styles need an explicit Vite entry');
    assert.match(read('src/bootstrap.rs'), /entry_points/, 'FWU-004: Inertia must request the explicit CSS entry');
    run('node', ['scripts/verify-foundation.mjs', 'ui']);
    const env = isolated();
    run('bun', ['install', '--frozen-lockfile', '--cwd', 'frontend'], working, env);
    run('bun', ['run', '--cwd', 'frontend', 'build'], working, env);
    run('bun', ['run', '--cwd', 'frontend', 'build:ssr'], working, env);
    run('cargo', ['build', '--locked', '--bins'], working, env);
    run('node', ['scripts/with-ssr.mjs', 'node', 'frontend/tests/framework-upgrade.mjs'], working, env);
  } else if (task === 'console') {
    assert.match(read('src/bin/console.rs'), /ExitCode::from\([^)]*\.exit_code\(\)/, 'FWU-005: preserve the command exit code');
    const env = isolated();
    run('cargo', ['build', '--locked', '--bin', 'console'], working, env);
    // No .env or assets exist in the copy. An unusable database URL catches an
    // accidental bootstrap on help, version, or invalid command arguments.
    env.DATABASE_URL = 'invalid://console-must-not-connect';
    env.APP_ENV = 'production';
    const binary = join(env.CARGO_TARGET_DIR, 'debug/console');
    run(binary, ['--help'], working, env);
    assert.match(run(binary, ['--version'], working, env), /\d+\.\d+\.\d+/);
    run(binary, ['not-a-real-command'], working, env, 2);
    run(binary, ['admin:access', '--not-a-real-option'], working, env, 2);
  } else {
    assert.match(read('UPGRADE.md'), /Suprnova.*4\.2\.0/, 'FWU-006: document the new framework release');
    assert.match(read('UPGRADE.md'), /revocation/i, 'FWU-006: document durable auth-revocation retry operation');
    const commands = [
      ['scripts/verify-foundation.mjs', 'build'], ['scripts/verify-foundation.mjs', 'database'],
      ['scripts/verify-foundation.mjs', 'accounts'], ['scripts/verify-foundation.mjs', 'ui'],
      ['scripts/verify-foundation.mjs', 'setup'], ['scripts/verify-provider-administration.mjs'],
      ['scripts/verify-paid-directory.mjs', 'directory'], ['scripts/verify-paid-directory.mjs', 'payments'],
      ...['editorial', 'administration', 'adoption', 'overview', 'seo'].map(group => ['scripts/verify-complete-starter.mjs', group]),
      ['scripts/verify-appearance.mjs'], ['.cairn/checks/verify-upgrade-guide.mjs'],
    ];
    for (const args of commands) run('node', args);
  }
  console.log(`sudus: ${requirement}: pass`);
} catch (error) {
  console.error(error.stack ?? error.message);
  if (requirement) console.log(`sudus: ${requirement}: fail`);
  process.exitCode = 1;
} finally {
  if (working) rmSync(working, { recursive: true, force: true });
}
