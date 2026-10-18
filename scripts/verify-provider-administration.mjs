import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const requirements = ['PAY-001', 'PAY-004', 'PAY-005', 'PAY-006', 'PAY-007', 'PAY-008'];
const secrets = ['SECRET_API_MARKER', 'SECRET_WEBHOOK_MARKER', 'SECRET_PADDLE_API', 'SECRET_PADDLE_WEBHOOK', 'REPLACEMENT_SECRET', 'WRONG_MODE_SECRET'];
let working;

function run(label, command, args, env, expected = 0) {
  console.log(`\n[providers] ${label}: ${command} ${args.join(' ')}`);
  const result = spawnSync(command, args, { cwd: working, env, detached: true, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, timeout: 30 * 60 * 1000 });
  // Kill survivors in this command's own process group even if a test crashes
  // before its finally block. Never signal unrelated application processes.
  if (result.pid) {
    try { process.kill(-result.pid, 'SIGKILL'); }
    catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
  if (result.error) throw result.error;
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`;
  for (const secret of secrets) assert.ok(!output.includes(secret), `${label} leaked a credential marker`);
  console.log(output);
  assert.equal(result.status, expected, `${label} exited ${result.status} (signal ${result.signal ?? 'none'})`);
  console.log(`[providers] ${label}: passed`);
  return result.stdout;
}

try {
  working = snapshot(source, 'directory-providers-');
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, {
    TMPDIR: working, APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'http://directory.test',
    APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test',
    DATABASE_URL: `sqlite://${join(working, 'providers.db')}`, DB_LOGGING: 'false', SESSION_SECURE: 'false',
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2', CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_TARGET_DIR: join(source, 'target'),
  });
  run('format', 'cargo', ['fmt', '--check'], env);
  run('build', 'cargo', ['build', '--locked', '--bins'], env);
  const contract = run('HTTP and persistence contract', 'cargo', ['test', '--locked', '--test', 'provider_administration', '--', '--nocapture'], env);
  const userId = contract.match(/BILLING_OPERATOR_ID=(\d+)/)?.[1];
  assert.ok(userId, 'The verified operator fixture was not produced');
  const consoleBin = join(env.CARGO_TARGET_DIR, 'debug/console');
  run('first administrator grant', consoleBin, ['admin:access', 'grant', '--user-id', userId], env);
  run('idempotent administrator grant', consoleBin, ['admin:access', 'grant', '--user-id', userId], env);
  run('unknown administrator rejected', consoleBin, ['admin:access', 'grant', '--user-id', '9223372036854775807'], env, 1);
  const keyProbe = ['test', '--locked', '--test', 'provider_key_policy', '--', '--nocapture'];
  run('restart with the original key', 'cargo', keyProbe, { ...env, BILLING_KEY_EXPECT: 'accept' });
  for (const [name, key] of [['missing', ''], ['default', Buffer.alloc(32).toString('base64url')], ['wrong', randomBytes(32).toString('base64url')]]) {
    const probeEnv = { ...env, APP_KEY: key, BILLING_KEY_EXPECT: 'reject' };
    if (name === 'missing') delete probeEnv.APP_KEY;
    run(`${name} deployment key`, 'cargo', keyProbe, probeEnv);
  }
  run('constructor failure and corrected case', 'cargo', ['test', '--locked', '--lib', 'billing::settings::tests::constructor_failure_preserves_configuration', '--', '--nocapture'],
    { ...env, DATABASE_URL: `sqlite://${join(working, 'constructor.db')}` });
  run('frontend install', 'bun', ['install', '--frozen-lockfile', '--cwd', 'frontend'], env);
  run('frontend build', 'bun', ['run', '--cwd', 'frontend', 'build'], env);
  run('frontend SSR build', 'bun', ['run', '--cwd', 'frontend', 'build:ssr'], env);
  run('browser administration journeys', 'node', ['scripts/with-ssr.mjs', 'node', 'frontend/tests/provider-administration.mjs'], { ...env, BILLING_OPERATOR_ID: userId, PROVIDER_ARTIFACT_DIR: join(source, 'target/provider-ui') });
  for (const requirement of requirements) console.log(`cairn: ${requirement}: pass`);
} catch (error) {
  console.error(error.stack ?? error.message);
  for (const requirement of requirements) console.log(`cairn: ${requirement}: fail`);
  process.exitCode = 1;
} finally {
  if (working) rmSync(working, { recursive: true, force: true });
}
