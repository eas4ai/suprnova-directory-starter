import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const read = path => readFileSync(join(root, path), 'utf8');

try {
  const guide = read('UPGRADE.md');
  const prose = guide.replace(/\s+/g, ' ');
  const manifest = read('Cargo.toml');
  const lock = read('Cargo.lock');
  const version = manifest.match(/^version = "([^"]+)"$/m)?.[1];
  assert.ok(version, 'Package version is missing');
  const application = lock.split('[[package]]').find(block => block.includes('\nname = "directory"\n'));
  assert.ok(application?.includes(`\nversion = "${version}"\n`), 'Locked application version differs');
  assert.equal(JSON.parse(read('frontend/package.json')).version, version, 'Frontend version differs');
  assert.ok(prose.includes(`package versions are ${version}`), 'Guide package versions differ');
  assert.ok(prose.includes('no new starter release tag is implied'), 'Do not relabel the existing starter release');

  const frameworkRevision = '6f6f83feae1dfde97d614d083625872987407780';
  const markdownRevision = '6f27ecae53babc0fd637c23b41bc5b244543c78e';
  for (const name of ['suprnova', 'suprnova-payments-stripe', 'suprnova-payments-paddle', 'suprnova-markdown']) {
    const revision = name === 'suprnova-markdown' ? markdownRevision : frameworkRevision;
    const dependency = manifest.split('\n').find(line => line.startsWith(`${name} = `));
    const isMarkdown = name === 'suprnova-markdown';
    const selector = isMarkdown ? `rev = "${revision}"` : 'tag = "v4.2.0"';
    assert.ok(dependency?.includes(selector), `${name} source selector differs`);
    const locked = lock.split('[[package]]').find(block => block.includes(`\nname = "${name}"\n`));
    const lockedSelector = isMarkdown ? `rev=${revision}` : 'tag=v4.2.0';
    assert.ok(locked?.includes(`?${lockedSelector}#${revision}`), `${name} lock revision differs`);
    assert.ok(guide.includes(`\`${revision}\``), `Guide omits ${name} revision`);
    if (name === 'suprnova') assert.ok(locked.includes('\nversion = "4.2.0"\n'), 'Framework version differs');
  }
  assert.ok(guide.includes('Suprnova 3.2.1 → 4.2.0'), 'Framework upgrade is missing');
  assert.ok(guide.includes('Markdown v0.4.0'), 'Markdown release is missing');
  const frontend = JSON.parse(read('frontend/package.json'));
  for (const name of ['@inertiajs/core', '@inertiajs/vue3', '@inertiajs/vite']) {
    assert.equal(frontend.dependencies?.[name] ?? frontend.devDependencies?.[name], '3.8.0', `${name} version differs`);
  }
  assert.ok(prose.includes('Inertia Vue, core and Vite packages are pinned to 3.8.0'), 'Guide omits Inertia versions');

  for (const boundary of [
    'APP_KEY', 'DATABASE_URL', 'DIRECTORY_MEDIA_ROOT', 'matching encryption key',
    'Stop web writers', 'scheduled payment/notification workers', 'WAL',
    'chrono::DateTime<Utc>', 'chrono::Utc::now()', 'RATE_LIMIT_DRIVER=redis',
    'RATE_LIMIT_ALLOW_MEMORY_IN_PRODUCTION=true', 'do not protect replicas',
    'MAIL_SMTP_HOST', 'MAIL_SMTP_PORT', 'MAIL_SMTP_USER', 'MAIL_SMTP_PASS', 'MAIL_FROM',
    'MAIL_SMTP_ENCRYPTION=starttls', 'plaintext SMTP', 'port private',
    'Restore the previous application', 'matching database, media and key backup',
    'do not replay payment events blindly',
    'RbacToSpatieLayout', 'CreateSessionsTable::new(SessionUserKey::Integer)',
    'signed application user IDs', 'sign in again', 'Old browser',
    'QUEUE_DRIVER=redis', 'QUEUE_REDIS_URL', 'QUEUE_REDIS_STREAM', 'QUEUE_REDIS_GROUP',
    'QUEUE_VISIBILITY_TIMEOUT_SECS=120', 'Leave `--tries` unset',
    'Suprnova::RevokeUserCredentials', 'Redis persistence', 'pending/delayed backlog',
    'do not let old workers consume new framework jobs', 'all 55 inherited requirements',
    'They do not establish PostgreSQL behavior', 'live charges',
    'src/app.css', 'private, no-store', 'ignores `public/hot`',
  ]) assert.ok(prose.includes(boundary), `Guide omits operating boundary: ${boundary}`);

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
    'git fetch upstream "$UPGRADE_REF"', 'git merge --no-ff FETCH_HEAD',
    'bun install --frozen-lockfile', 'bun run build:ssr', 'cargo build --locked --release --bins',
    './target/release/directory migrate', './target/release/directory serve --no-migrate',
    'node --env-file=.env frontend/bootstrap/ssr/ssr.js',
    './target/release/console billing:reconcile --status --limit 100',
    './target/release/console notifications:deliver --status --limit 100',
    './target/release/directory queue:work --queue default --visibility-timeout 120',
  ]) assert.ok(commands.includes(command), `Guide omits command: ${command}`);
  console.log('Dependency pins, migration/worker boundaries, backup link and shell syntax verified.');
  console.log('cairn: UPG-001: pass');
} catch (error) {
  console.error(error.stack ?? error.message);
  console.log('cairn: UPG-001: fail');
  process.exitCode = 1;
}
