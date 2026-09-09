import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

// The parent creates and cleans the source snapshot and supplies its build cache.
// This helper owns only new files inside that explicitly disposable directory.
assert.equal(process.env.ADOPTION_INSTALL_DISPOSABLE, '1', 'Run through the adoption verifier in a disposable source snapshot.');
const working = process.cwd();
const database = join(working, 'adoption-install.db');
assert.ok(!existsSync(database), 'Clean-install database must not already exist.');
assert.ok(!existsSync(join(working, '.env')), 'Clean install must not inherit an operator .env.');
const env = { ...process.env, APP_ENV: 'test', APP_DEBUG: 'false',
  DATABASE_URL: `sqlite://${database}`, APP_KEY: randomBytes(32).toString('base64url'),
  DIRECTORY_MEDIA_ROOT: join(working, 'adoption-install-media'),
  BACKUP_DIR: join(working, 'adoption-install-backup'),
  RESTORE_DIR: join(working, 'adoption-install-restored'),
};
const license = readFileSync(join(working, 'LICENSE'), 'utf8');
const notices = readFileSync(join(working, 'THIRD_PARTY_NOTICES.md'), 'utf8');
for (const required of ['MIT License', 'Copyright (c) 2026 Shawn McAllister',
  'Permission is hereby granted, free of charge',
  'The above copyright notice and this permission notice shall be included',
  'THE SOFTWARE IS PROVIDED "AS IS"']) assert.ok(license.includes(required), `Missing MIT license term: ${required}`);
assert.ok(notices.includes(license.trim()), 'Pulsar full MIT notice must be preserved');
assert.ok(notices.includes('Larafast Directories') && notices.includes('behavioral reference'), 'Purchased reference boundary must be documented');
for (const file of ['Cargo.lock', 'frontend/bun.lock']) assert.ok(readFileSync(join(working, file)).length > 0, `Missing dependency provenance lock: ${file}`);
const readme = readFileSync(join(working, 'README.md'), 'utf8');
function run(command, args, overrides = {}, expectedSuccess = true) {
  console.log(`[adoption install] ${command} ${args[0] ?? ''}`);
  const result = spawnSync(command, args, { cwd: working, env: { ...env, ...overrides }, encoding: 'utf8', timeout: 120_000, maxBuffer: 8 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (expectedSuccess) assert.equal(result.status, 0, `${command} failed: ${result.stderr} ${result.stdout}`);
  else assert.ok(result.status !== 0 && result.status !== null, `Expected a command refusal: ${result.stdout}`);
  return result.stdout;
}
function block(name, overrides) {
  const marker = `<!-- adoption:${name} -->`;
  const start = readme.indexOf(marker);
  assert.ok(start >= 0, `Missing README block ${marker}`);
  const match = readme.slice(start + marker.length).match(/^\s*```sh\n([\s\S]*?)\n```/);
  assert.ok(match, `Missing executable shell block for ${name}`);
  return run('bash', ['-euc', match[1]], overrides);
}
function consoleCommand(args, expectedSuccess = true, overrides = {}) {
  return run('cargo', ['run', '--locked', '--bin', 'console', '--', ...args], overrides, expectedSuccess);
}
function dump(path = database) {
  return run('python3', ['-c', 'import sqlite3,sys; db=sqlite3.connect(sys.argv[1]); print("\\n".join(db.iterdump()))', path]);
}
const migration = ['run', '--locked', '--bin', 'directory', '--', 'migrate'];
run('cargo', migration);
const firstMigration = dump();
run('cargo', migration);
assert.equal(dump(), firstMigration, 'Repeated migrations must preserve schema and history.');
block('seed');
const seeded = dump();
block('seed');
const reseeded = dump();
// SQLite allocates AUTOINCREMENT IDs even when directory:categories encounters
// an existing slug and INSERT ... ON CONFLICT DO NOTHING preserves the row.
// Ignore only this table's internal next-ID counter, never schema/business data
// or other counters. Backup/restore comparisons below remain byte-exact.
const categorySequence = /^INSERT INTO "sqlite_sequence" VALUES\('listing_categories',\d+\);$/gm;
assert.equal(reseeded.replace(categorySequence, ''), seeded.replace(categorySequence, ''),
  'Repeated documented seeds must preserve schema, business rows and other sequence counters.');
// These isolated command fixtures are not a documented route to verify real users.
run('python3', ['-c', `import sqlite3,sys
with sqlite3.connect(sys.argv[1]) as db:
 db.execute("INSERT INTO users (id,name,email,password,email_verified_at) VALUES (900001,'Recovery fixture','recovery@example.test','unusable-test-hash',CURRENT_TIMESTAMP)")
 db.execute("INSERT INTO users (id,name,email,password) VALUES (900002,'Unverified fixture','unverified@example.test','unusable-test-hash')")`, database]);
consoleCommand(['admin:access', 'grant', '--user-id', '900001']);
consoleCommand(['admin:access', 'grant', '--user-id', '900001']);
consoleCommand(['admin:access', 'revoke', '--user-id', '900001']);
consoleCommand(['admin:access', 'grant', '--user-id', '900001']);
consoleCommand(['admin:access', 'grant', '--user-id', '900002'], false);
consoleCommand(['admin:access', 'grant', '--user-id', '999999'], false);
consoleCommand(['billing:reconcile', '--limit', '0'], false);
consoleCommand(['notifications:deliver', '--limit', '0'], false);
block('operations');
mkdirSync(env.DIRECTORY_MEDIA_ROOT, { recursive: true });
const media = randomBytes(64);
writeFileSync(join(env.DIRECTORY_MEDIA_ROOT, 'restore-fixture.bin'), media);
const beforeBackup = dump();
block('backup');
block('restore');
const restoredDatabase = join(env.RESTORE_DIR, 'database.db');
assert.equal(dump(restoredDatabase), beforeBackup, 'Restored database differs from backup source.');
assert.deepEqual(readFileSync(join(env.RESTORE_DIR, 'media/restore-fixture.bin')), media);
assert.equal(readFileSync(join(env.RESTORE_DIR, 'APP_KEY'), 'utf8'), env.APP_KEY);
const restored = { DATABASE_URL: `sqlite://${restoredDatabase}`, DIRECTORY_MEDIA_ROOT: join(env.RESTORE_DIR, 'media'), APP_KEY: readFileSync(join(env.RESTORE_DIR, 'APP_KEY'), 'utf8') };
run('cargo', migration, restored);
consoleCommand(['admin:access', 'grant', '--user-id', '900001'], true, restored);
block('operations', restored);
assert.ok(resolve(restoredDatabase).startsWith(`${working}/`));
console.log('Documented migrations, seeds, operator commands and SQLite/media/key restoration passed.');
