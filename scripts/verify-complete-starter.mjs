import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const task = process.argv[2];
const source = resolve(process.argv[3] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const groups = {
  editorial: Array.from({ length: 6 }, (_, index) => `CNT-${String(index + 1).padStart(3, '0')}`),
  administration: Array.from({ length: 3 }, (_, index) => `ADM-${String(index + 1).padStart(3, '0')}`),
  adoption: Array.from({ length: 5 }, (_, index) => `KIT-${String(index + 1).padStart(3, '0')}`),
};
const requirements = groups[task];
let working;
function run(command, args, env) {
  console.log(`[${task}] ${command} ${args.join(' ')}`);
  const result = spawnSync(command, args, { cwd: working, env, detached: true, stdio: 'inherit', timeout: 30 * 60 * 1000 });
  if (result.pid) {
    try { process.kill(-result.pid, 'SIGKILL'); }
    catch (error) { if (error.code !== 'ESRCH') throw error; }
  }
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} exited ${result.status} (signal ${result.signal ?? 'none'})`);
}
try {
  assert.ok(requirements, 'Choose editorial, administration or adoption verification.');
  assert.ok(requirements, 'Unknown verification group.');
  working = snapshot(source, 'directory-complete-');
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, {
    APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'https://catalog.example.test', APP_NAME: 'Directory',
    APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test', BILLING_CHECKOUT_MODE: 'test',
    DATABASE_URL: `sqlite://${join(working, `${task}.db`)}`, DB_LOGGING: 'false', SESSION_SECURE: 'false',
    DIRECTORY_MEDIA_ROOT: join(working, 'storage/private/directory'), TMPDIR: working,
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2', CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: join(source, 'target'),
  });
  run('cargo', ['fmt', '--check'], env);
  run('cargo', ['build', '--locked', '--bins'], env);
  run('bun', ['install', '--frozen-lockfile', '--cwd', 'frontend'], env);
  run('bun', ['run', '--cwd', 'frontend', 'build'], env);
  run('bun', ['run', '--cwd', 'frontend', 'build:ssr'], env);
  if (task === 'adoption') {
    run('cargo', ['test', '--locked', '--test', 'demo_seed', '--', '--nocapture'], env);
    run('node', ['scripts/verify-adoption-install.mjs'], { ...env, ADOPTION_INSTALL_DISPOSABLE: '1' });
  }
  run('node', ['scripts/with-ssr.mjs', 'cargo', 'test', '--locked', '--test', `${task}_workflows`, '--', '--nocapture'], env);
  run('node', ['scripts/with-ssr.mjs', 'node', `frontend/tests/${task}-workflows.mjs`], { ...env, [`${task.toUpperCase()}_ARTIFACT_DIR`]: join(source, `target/${task}-ui`) });
  for (const requirement of requirements) console.log(`cairn: ${requirement}: pass`);
} catch (error) {
  console.error(error.stack ?? error.message);
  for (const requirement of requirements ?? []) console.log(`cairn: ${requirement}: fail`);
  process.exitCode = 1;
} finally {
  if (working) rmSync(working, { recursive: true, force: true });
}
