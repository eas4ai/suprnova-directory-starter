import assert from 'node:assert/strict';
import { Database } from 'bun:sqlite';

const seedProbe = process.argv[3] === '--seed-probe';
const db = new Database(process.argv[2], { readonly: !seedProbe, create: false });
try {
  const schema = db.query("SELECT type, name, tbl_name, sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type, name").all();
  const tables = new Set(schema.filter(row => row.type === 'table').map(row => row.name));
  const required = {
    users: ['id', 'name', 'email', 'password', 'email_verified_at', 'remember_token', 'created_at', 'updated_at'],
    sessions: ['id', 'payload', 'last_activity'],
    remember_tokens: ['id', 'user_id', 'token_hash'],
    auth_flow_tokens: ['id', 'token_hash'],
    seaql_migrations: ['version', 'applied_at'],
  };
  for (const [table, columns] of Object.entries(required)) {
    assert.ok(tables.has(table), `Missing account table: ${table}`);
    const found = new Set(db.query(`PRAGMA table_info("${table}")`).all().map(row => row.name));
    for (const column of columns) assert.ok(found.has(column), `Missing account column: ${table}.${column}`);
  }
  const migrations = db.query('SELECT version, applied_at FROM seaql_migrations ORDER BY version').all();
  for (const version of [
    'm20240101_000001_create_users_table',
    'm20240101_000002_create_sessions_table',
    'm20240101_000003_create_remember_tokens_table',
    'm20240101_000004_create_auth_flow_tokens_table',
  ]) assert.ok(migrations.some(row => row.version === version), `Missing migration receipt: ${version}`);
  if (seedProbe) {
    db.query('INSERT INTO users (name, email, password) VALUES (?, ?, ?)')
      .run('Migration probe', 'migration-probe@example.test', 'not-a-login-hash');
  }
  const probe = db.query('SELECT name, email, password FROM users WHERE email = ?').all('migration-probe@example.test');
  console.log(JSON.stringify({ schema, migrations, probe }));
} finally {
  db.close();
}
