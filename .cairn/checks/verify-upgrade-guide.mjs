import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '../..'));
const read = path => readFileSync(join(root, path), 'utf8');

try {
  const guide = read('UPGRADE.md');
  const manifest = read('Cargo.toml');
  const lock = read('Cargo.lock');
  const version = manifest.match(/^version = "([^"]+)"$/m)?.[1];
  assert.ok(version, 'Package version is missing');
  const application = lock.split('[[package]]').find(block => block.includes('\nname = "directory"\n'));
  assert.ok(application?.includes(`\nversion = "${version}"\n`), 'Locked application version differs');
  assert.equal(JSON.parse(read('frontend/package.json')).version, version, 'Frontend version differs');
  assert.ok(guide.includes(`Release \`v${version}\``), 'Guide release differs from the package');
  assert.ok(guide.includes(`versions are ${version}`), 'Guide package versions differ');

  const frameworkRevision = '2bd4bd53d04fa4581fdbb152ebd70cfb682b342c';
  const markdownRevision = '565c2205b1d1d77aa05a29247f3a2416abc60791';
  for (const name of ['suprnova', 'suprnova-payments-stripe', 'suprnova-payments-paddle', 'suprnova-markdown']) {
    const revision = name === 'suprnova-markdown' ? markdownRevision : frameworkRevision;
    const dependency = manifest.split('\n').find(line => line.startsWith(`${name} = `));
    assert.ok(dependency?.includes(`rev = "${revision}"`), `${name} revision differs`);
    const locked = lock.split('[[package]]').find(block => block.includes(`\nname = "${name}"\n`));
    assert.ok(locked?.includes(`?rev=${revision}#${revision}`), `${name} lock revision differs`);
    assert.ok(guide.includes(`\`${revision}\``), `Guide omits ${name} revision`);
    if (name === 'suprnova') assert.ok(locked.includes('\nversion = "3.2.1"\n'), 'Framework version differs');
  }
  assert.ok(guide.includes('Suprnova from 2.0.0 to 3.2.1'), 'Framework upgrade is missing');

  for (const boundary of [
    'APP_KEY', 'DATABASE_URL', 'DIRECTORY_MEDIA_ROOT', 'matching encryption key',
    'Stop web writers', 'scheduled payment/notification workers', 'WAL',
    'chrono::DateTime<Utc>', 'chrono::Utc::now()', 'RATE_LIMIT_DRIVER=redis',
    'RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true', 'do not protect replicas',
    'MAIL_SMTP_HOST', 'MAIL_SMTP_PORT', 'MAIL_SMTP_USER', 'MAIL_SMTP_PASS', 'MAIL_FROM',
    'MAIL_SMTP_ENCRYPTION=starttls', 'plaintext SMTP', 'port private',
    'Restore the previous application', 'matching database, media and key backup',
    'do not replay payment events blindly',
  ]) assert.ok(guide.includes(boundary), `Guide omits operating boundary: ${boundary}`);

  assert.ok(guide.includes('(README.md#production-operation)'), 'Backup procedure link is missing');
  assert.ok(read('README.md').includes('## Production operation'), 'Backup link target is missing');
  const blocks = [...guide.matchAll(/```(?:bash|sh)\n([\s\S]*?)```/g)].map(match => match[1]);
  assert.ok(blocks.length >= 5, 'Expected merge, build, migrate, startup and worker examples');
  for (const [index, block] of blocks.entries()) {
    const result = spawnSync('bash', ['-n'], { input: block, encoding: 'utf8', timeout: 10000 });
    if (result.error) throw result.error;
    assert.equal(result.status, 0, `Shell example ${index + 1} is invalid: ${result.stderr}`);
    assert.ok(!/migrate:fresh|db:seed|key:generate|key:regen|\bcp\s+\.env\.example\s+\.env\b/.test(block),
      `Shell example ${index + 1} contains a destructive fresh-install command`);
  }
  const commands = blocks.join('\n');
  for (const command of [
    `git fetch upstream refs/tags/v${version}`, 'git merge --no-ff FETCH_HEAD',
    'bun install --frozen-lockfile', 'bun run build:ssr', 'cargo build --locked --release --bins',
    './target/release/directory migrate', './target/release/directory serve --no-migrate',
    'node --env-file=.env frontend/bootstrap/ssr/ssr.js',
    './target/release/console billing:reconcile --status --limit 100',
    './target/release/console notifications:deliver --status --limit 100',
  ]) assert.ok(commands.includes(command), `Guide omits command: ${command}`);
  console.log('Release pins, operating boundaries, backup link and shell syntax verified.');
  console.log('cairn: UPG-001: pass');
} catch (error) {
  console.error(error.stack ?? error.message);
  console.log('cairn: UPG-001: fail');
  process.exitCode = 1;
}
