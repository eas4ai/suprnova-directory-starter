import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[3] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const task = process.argv[2];
const requirements = { build: 'FND-001', database: 'FND-002', accounts: 'FND-003', ui: 'FND-004', setup: 'FND-005' };
const requirement = requirements[task];

function run(command, args, cwd, options = {}) {
  console.log(`\n[${task}] ${command} ${args.join(' ')}`);
  const result = spawnSync(command, args, {
    cwd,
    env: {
      ...process.env,
      CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2',
      CARGO_PROFILE_DEV_DEBUG: '0',
      CARGO_TARGET_DIR: join(source, 'target'),
    },
    stdio: 'inherit',
    timeout: 30 * 60 * 1000,
    ...options,
  });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${command} failed (exit ${result.status}, signal ${result.signal ?? 'none'})`);
  console.log(`[${task}] command passed`);
  return result.stdout;
}

function verifyPin(root) {
  const metadata = run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], root, {
    encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'],
  });
  const app = JSON.parse(metadata).packages.find(pkg => pkg.name === 'directory');
  const revision = '107e6e7a122d5145160ea1547ca90ddc37459c27';
  const selectedSource = `git+https://github.com/eas4ai/suprnova.git?rev=${revision}`;
  for (const name of ['suprnova', 'suprnova-payments-stripe', 'suprnova-payments-paddle']) {
    assert.equal(app?.dependencies.find(dependency => dependency.name === name)?.source,
      selectedSource, `${name} must use the agreed repaired framework revision`);
  }
  assert.deepEqual(app.targets.filter(target => target.kind.includes('bin')).map(target => target.name).sort(), ['console', 'directory']);
  const lock = readFileSync(join(root, 'Cargo.lock'), 'utf8');
  const frameworkSources = [...lock.matchAll(/^source = "(git\+https:\/\/github\.com\/eas4ai\/suprnova\.git[^"\n]+)"$/gm)].map(match => match[1]);
  assert.ok(frameworkSources.length >= 3, 'The lockfile must include the framework and payment adapters');
  assert.ok(frameworkSources.every(value => value === `${selectedSource}#${revision}`),
    'Every locked Suprnova package must resolve the exact agreed revision');
}

function build(root) {
  const lockPaths = ['Cargo.lock', 'frontend/bun.lock'];
  const before = lockPaths.map(path => readFileSync(join(root, path)));
  verifyPin(root);
  run('cargo', ['build', '--locked', '--bins'], root);
  run('bun', ['install', '--frozen-lockfile'], join(root, 'frontend'));
  run('bun', ['run', 'build'], join(root, 'frontend'));
  run('bun', ['run', 'build:ssr'], join(root, 'frontend'));
  lockPaths.forEach((path, index) => assert.deepEqual(readFileSync(join(root, path)), before[index], `${path} changed during a locked install`));
  assert.ok(existsSync(join(root, 'public/assets/.vite/manifest.json')), 'Client build manifest is missing');
  assert.ok(existsSync(join(root, 'frontend/bootstrap/ssr/ssr.js')), 'SSR bundle is missing');
}

function database(root) {
  const databasePath = join(root, 'database.db');
  assert.ok(!existsSync(databasePath), 'Migration check must start with an empty database');
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME', 'TMPDIR'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, {
    APP_ENV: 'test',
    APP_DEBUG: 'false',
    DATABASE_URL: `sqlite://${databasePath}`,
    DB_LOGGING: 'false',
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2',
    CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_TARGET_DIR: join(source, 'target'),
  });
  const command = ['run', '--locked', '--bin', 'directory', '--', 'migrate'];
  const inspect = (args = []) => run('bun', ['scripts/check-schema.mjs', databasePath, ...args], root, {
    env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'],
  });
  console.log('[database] first migration run');
  run('cargo', command, root, { env });
  const first = JSON.parse(inspect(['--seed-probe']));
  console.log('[database] repeat migration run');
  run('cargo', command, root, { env });
  assert.deepEqual(JSON.parse(inspect()), first, 'Repeated migrations changed the schema, migration history or existing account data');
}

function accounts(root, suite = 'foundation_accounts') {
  const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME', 'TMPDIR'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
  Object.assign(env, {
    TMPDIR: root, APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'http://directory.test',
    MAIL_FROM: 'test@example.test', DATABASE_URL: `sqlite://${join(root, 'accounts.db')}`,
    DB_LOGGING: 'false', SESSION_SECURE: 'false',
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2', CARGO_PROFILE_DEV_DEBUG: '0',
    CARGO_TARGET_DIR: join(source, 'target'),
  });
  run('cargo', ['test', '--locked', '--test', suite, '--', '--nocapture'], root, { env });
  return env;
}

function setup(root) {
  run('node', ['frontend/tests/foundation-setup.mjs'], root);
}

function ui(root) {
  run('bun', ['install', '--frozen-lockfile'], join(root, 'frontend'));
  run('bun', ['run', 'build'], join(root, 'frontend'));
  const env = accounts(root, 'foundation_ui');
  env.FOUNDATION_ARTIFACT_DIR = join(source, 'target/foundation-ui');
  run('node', ['frontend/tests/foundation-ui.mjs'], root, { env });
}

if (!requirement) {
  console.error(`Foundation mechanism is not implemented for: ${task ?? '(missing task)'}`);
  process.exitCode = 1;
} else {
  let working;
  try {
    working = snapshot(source, 'directory-foundation-');
    ({ build, database, accounts, ui, setup })[task](working);
    console.log(`cairn: ${requirement}: pass`);
  } catch (error) {
    console.error(error.stack ?? error.message);
    console.log(`cairn: ${requirement}: fail`);
    process.exitCode = 1;
  } finally {
    if (working) rmSync(working, { recursive: true, force: true });
  }
}
