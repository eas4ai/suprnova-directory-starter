import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const source = resolve(process.argv[3] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const task = process.argv[2];
const requirements = { build: 'FND-001' };
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

function snapshot(root) {
  const result = spawnSync('git', ['ls-files', '-z', '--cached'], {
    cwd: root, encoding: 'utf8', timeout: 10_000,
  });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `Cannot inventory the checkout: ${result.stderr}`);
  const destination = mkdtempSync(join(tmpdir(), 'directory-foundation-'));
  const roots = ['Cargo.toml', 'Cargo.lock', 'src/', 'cmd/', 'frontend/', 'lang/', 'scripts/', 'tests/', 'README.md', '.env.example'];
  try {
    for (const file of new Set(result.stdout.split('\0').filter(Boolean))) {
      if (!roots.some(root => root.endsWith('/') ? file.startsWith(root) : file === root)) continue;
      // A local environment or generated asset never belongs in a clean install.
      if (file.split('/').some(part => part === '.env' || part === 'node_modules' || part === 'target')) continue;
      const target = join(destination, file);
      mkdirSync(dirname(target), { recursive: true });
      cpSync(join(root, file), target);
    }
    return destination;
  } catch (error) {
    rmSync(destination, { recursive: true, force: true });
    throw error;
  }
}

function verifyPin(root) {
  const metadata = run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], root, {
    encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'],
  });
  const app = JSON.parse(metadata).packages.find(pkg => pkg.name === 'directory');
  const framework = app?.dependencies.find(dependency => dependency.name === 'suprnova');
  assert.equal(framework?.source, 'git+https://github.com/eas4ai/suprnova.git?tag=v1.3.7', 'The foundation must use Suprnova v1.3.7');
  assert.deepEqual(app.targets.filter(target => target.kind.includes('bin')).map(target => target.name).sort(), ['console', 'directory']);
  const lock = readFileSync(join(root, 'Cargo.lock'), 'utf8');
  assert.match(lock, /name = "suprnova"\nversion = "[^"\n]+"\nsource = "git\+https:\/\/github\.com\/eas4ai\/suprnova\.git\?tag=v1\.3\.7#[a-f0-9]+"/, 'The lockfile must resolve the selected framework tag');
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

if (!requirement) {
  console.error(`Foundation mechanism is not implemented for: ${task ?? '(missing task)'}`);
  process.exitCode = 1;
} else {
  let working;
  try {
    working = snapshot(source);
    build(working);
    console.log(`cairn: ${requirement}: pass`);
  } catch (error) {
    console.error(error.stack ?? error.message);
    console.log(`cairn: ${requirement}: fail`);
    process.exitCode = 1;
  } finally {
    if (working) rmSync(working, { recursive: true, force: true });
  }
}
