import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { randomBytes } from 'node:crypto';
import { readFileSync, writeFileSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { snapshot } from './verification-workspace.mjs';

const source = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const cases = [
  ['PAY-003/PAY-011 signatures', 'events.rs', 'adapter.verify(&context).is_ok()', 'true'],
  ['PAY-010 retry correlation', 'checkout.rs', '(row.provider == "stripe").then(|| row.id.clone())', '(row.provider == "stripe").then(|| uuid::Uuid::new_v4().to_string())'],
  ['PAY-012 persisted fulfillment', 'fulfillment.rs', 'subscription_ref: Set(row.subscription_ref)', 'subscription_ref: sea_orm::ActiveValue::NotSet'],
  ['PAY-013 full refunds', 'fulfillment.rs', 'let refunded = observations', 'let refunded = false && observations'],
  ['PAY-014 bounded recovery', 'reconcile.rs', 'if !(1..=100).contains(&limit)', 'if !(1..=1000).contains(&limit)'],
  ['PAY-015 retained signing keys', 'events.rs', 'if let Some(purchase) = &candidate', 'if let Some(purchase) = candidate.as_ref().filter(|_| false)'],
  ['PAY-016 SDK request expansion', 'gateway.rs', 'expand: ["line_items"]', 'expand: ["customer"]', 'payment_sdk_wire'],
];
for (const [name, file, before, after, suite = 'paid_lifecycle'] of cases) {
  const working = snapshot(source, 'paid-negative-');
  try {
    const path = join(working, 'src/billing', file);
    const original = readFileSync(path, 'utf8');
    assert.ok(original.includes(before), `${name}: source mutation is stale`);
    writeFileSync(path, original.replace(before, after));
    const env = Object.fromEntries(['PATH', 'HOME', 'CARGO_HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
    Object.assign(env, { APP_ENV: 'test', APP_DEBUG: 'false', APP_URL: 'http://directory.test',
      APP_KEY: randomBytes(32).toString('base64url'), MAIL_FROM: 'test@example.test', BILLING_CHECKOUT_MODE: 'live',
      DATABASE_URL: `sqlite://${join(working, 'paid.db')}`, DIRECTORY_MEDIA_ROOT: join(working, 'private-media'),
      DB_LOGGING: 'false', SESSION_SECURE: 'false', TMPDIR: working, CARGO_BUILD_JOBS: '2',
      CARGO_PROFILE_DEV_DEBUG: '0', CARGO_INCREMENTAL: '0', CARGO_TARGET_DIR: join(source, 'target') });
    const result = spawnSync('cargo', ['test', '--locked', '--test', suite, '--', '--nocapture'], {
      cwd: working, env, encoding: 'utf8', timeout: 600000, detached: true, maxBuffer: 20 * 1024 * 1024,
    });
    if (result.pid) {
      try { process.kill(-result.pid, 'SIGKILL'); } catch (error) { if (error.code !== 'ESRCH') throw error; }
    }
    if (result.error) throw result.error;
    const output = result.stdout + result.stderr;
    assert.equal(result.status, 101, `${name}: expected runtime test failure, got ${result.status}`);
    assert.match(output, /test .* \.\.\. FAILED/);
    const marker = `panicked at tests/${suite}.rs`;
    assert.ok(output.includes(marker), `${name}: compilation, setup and timeouts are not proof of failure sensitivity\n${output.slice(-3000)}`);
    console.log(`${name} rejected: ${output.slice(output.indexOf(marker), output.indexOf(marker) + 400)}`);
  } finally {
    rmSync(working, { recursive: true, force: true });
  }
}
