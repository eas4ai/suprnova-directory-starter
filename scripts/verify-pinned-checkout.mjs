#!/usr/bin/env node
// Verify the dependency's committed checkout implementation, never a patched cache.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const revision = '107e6e7a122d5145160ea1547ca90ddc37459c27';
const source = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
const cargoHome = resolve(process.env.CARGO_HOME ?? join(homedir(), '.cargo'));
const environment = Object.fromEntries(['PATH', 'HOME', 'RUSTUP_HOME'].filter(key => process.env[key]).map(key => [key, process.env[key]]));
Object.assign(environment, {
  CARGO_HOME: cargoHome,
  CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? '2',
  CARGO_PROFILE_DEV_DEBUG: '0', CARGO_INCREMENTAL: '0', CARGO_TERM_COLOR: 'never',
  CARGO_TARGET_DIR: resolve(process.env.CARGO_TARGET_DIR ?? join(source, 'target')),
  GIT_OPTIONAL_LOCKS: '0',
});
const required = {
  'suprnova-payments-stripe': [
    'idempotency_tests::hosted_checkout_wire_preserves_metadata_line_items_and_flags',
    'idempotency_tests::subscription_checkout_wire_preserves_metadata_line_items_and_flags',
    'idempotency_tests::elements_checkout_wire_preserves_metadata',
    'idempotency_tests::checkout_wire_omits_absent_metadata_and_disabled_managed_payments',
    'idempotency_tests::idempotency_keys_are_request_headers_on_every_supported_mutation',
    'idempotency_tests::invalid_idempotency_keys_are_rejected_before_network_io',
    'idempotency_tests::elements_session_status_retrieves_payment_intent_and_only_succeeded_is_paid',
    'idempotency_tests::elements_session_status_preserves_provider_errors',
    'idempotency_tests::session_status_rejects_invalid_identifiers_before_network',
    'checkout::tests::complete_paid_session_maps_with_payment_ref_and_total',
    'checkout::tests::complete_unpaid_session_maps_paid_false',
  ],
  'suprnova-payments-paddle': [
    'checkout::tests::checkout_forwards_correlation_metadata',
    'checkout::tests::checkout_metadata_uses_existing_string_map_policy',
    'checkout::tests::checkout_does_not_mislabel_customer_id_as_auth_token',
    'checkout::tests::retrieval_maps_each_transaction_status',
    'checkout::tests::retrieval_rejects_invalid_totals_and_provider_errors',
    'checkout::tests::retrieval_rejects_malformed_transaction_identifiers_before_io',
    'checkout::tests::create_timeout_reports_an_uncertain_outcome',
    'checkout::tests::retrieval_timeout_is_bounded',
    'idempotency_tests::unsupported_idempotency_keys_fail_before_paddle_mutations',
  ],
};
let temporary;
let active;
let interrupted;
function killGroup(child) {
  if (!child?.pid) return;
  try { process.kill(-child.pid, 'SIGKILL'); }
  catch (error) { if (error.code !== 'ESRCH') throw error; }
}
function interrupt(signal) {
  interrupted = signal;
  if (active) killGroup(active);
}
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => interrupt(signal));

async function run(command, args, { cwd = source, timeout = 30_000, quiet = false } = {}) {
  assert.ok(!interrupted, `Interrupted by ${interrupted}`);
  if (!quiet) console.log(`[pinned-checkout] ${command} ${args.join(' ')}`);
  return await new Promise((accept, reject) => {
    const child = spawn(command, args, { cwd, env: environment, detached: true, stdio: ['ignore', 'pipe', 'pipe'] });
    active = child;
    let output = ''; let errorOutput = ''; let timedOut = false; let tooLarge = false;
    const timer = setTimeout(() => { timedOut = true; killGroup(child); }, timeout);
    child.stdout.on('data', chunk => {
      output += chunk.toString();
      if (!quiet) process.stdout.write(chunk);
      if (output.length > 16 * 1024 * 1024) { tooLarge = true; killGroup(child); }
    });
    child.stderr.on('data', chunk => { errorOutput = (errorOutput + chunk.toString()).slice(-32_768); if (!quiet) process.stderr.write(chunk); });
    child.on('error', error => { clearTimeout(timer); active = undefined; reject(error); });
    child.on('close', (code, signal) => {
      clearTimeout(timer); killGroup(child); active = undefined;
      if (timedOut || tooLarge || interrupted || code !== 0) {
        reject(new Error(`${command} failed: ${timedOut ? 'timeout' : tooLarge ? 'output limit' : interrupted ?? signal ?? code}\n${errorOutput}`));
      } else accept(output);
    });
  });
}

async function assertClean(checkout) {
  const head = (await run('git', ['-C', checkout, 'rev-parse', 'HEAD'], { quiet: true })).trim();
  assert.equal(head, revision, 'Dependency checkout does not match the pinned revision.');
  const status = await run('git', ['-C', checkout, 'status', '--porcelain=v1', '-z', '--untracked-files=all'], { quiet: true });
  // Cargo writes this completion marker after unpacking. It is metadata, not source,
  // and git archive cannot include it. Every other dirty/untracked path is rejected.
  assert.deepEqual(status.split('\0').filter(entry => entry && entry !== '?? .cargo-ok'), [], 'Pinned dependency source is dirty.');
}
async function findCheckout() {
  const root = join(cargoHome, 'git', 'checkouts');
  assert.ok(existsSync(root), 'Cargo dependency cache is missing; fetch the locked application dependencies first.');
  const candidates = [];
  for (const repo of readdirSync(root, { withFileTypes: true })) {
    if (!repo.isDirectory() || !repo.name.startsWith('suprnova-')) continue;
    for (const checkout of readdirSync(join(root, repo.name), { withFileTypes: true })) {
      if (!checkout.isDirectory() || !revision.startsWith(checkout.name)) continue;
      const directory = join(root, repo.name, checkout.name);
      if (existsSync(join(directory, 'crates/suprnova-payments-stripe/Cargo.toml')) && existsSync(join(directory, 'crates/suprnova-payments-paddle/Cargo.toml'))) candidates.push(directory);
    }
  }
  assert.ok(candidates.length, `Pinned Suprnova revision ${revision} is absent from the Cargo cache.`);
  const checkout = candidates.sort()[0];
  await assertClean(checkout);
  return checkout;
}

try {
  const manifest = readFileSync(join(source, 'Cargo.toml'), 'utf8');
  const lockfile = readFileSync(join(source, 'Cargo.lock'), 'utf8');
  for (const name of ['suprnova', ...Object.keys(required)]) {
    const line = manifest.split('\n').find(line => line.startsWith(`${name} =`));
    assert.ok(line?.includes(`rev = "${revision}"`) && line.includes('https://github.com/eas4ai/suprnova.git'), `${name} must use the recorded checkout revision.`);
    const block = lockfile.split('[[package]]').find(block => block.includes(`\nname = "${name}"\n`));
    assert.ok(block?.includes(`git+https://github.com/eas4ai/suprnova.git?rev=${revision}#${revision}`), `${name} lockfile source must match the recorded revision.`);
  }
  const checkout = await findCheckout();
  const scratch = resolve(process.env.FOUNDATION_SCRATCH_DIR ?? join(source, '../scratchpads'));
  mkdirSync(scratch, { recursive: true }); temporary = mkdtempSync(join(scratch, 'pinned-checkout-'));
  const working = join(temporary, 'repository'); mkdirSync(working);
  environment.TMPDIR = temporary;
  const archive = join(temporary, 'source.tar');
  await run('git', ['-C', checkout, 'archive', '--format=tar', `--output=${archive}`, revision]);
  await run('tar', ['-xf', archive, '-C', working]);
  assert.ok(existsSync(join(working, 'Cargo.lock')), 'Pinned revision must contain a committed dependency lockfile.');
  console.log(`[pinned-checkout] committed source ${revision}; Cargo target ${environment.CARGO_TARGET_DIR}`);
  for (const [name, names] of Object.entries(required)) {
    const base = ['test', '--locked', '-p', name, '--lib'];
    const listing = await run('cargo', [...base, '--', '--list', '--format', 'terse'], { cwd: working, timeout: 30 * 60_000 });
    for (const test of names) assert.ok(listing.split('\n').includes(`${test}: test`), `Missing required test ${name}::${test}`);
    for (const filter of ['checkout::tests::', 'idempotency_tests::']) {
      const output = await run('cargo', [...base, filter, '--', '--nocapture', '--test-threads=1'], { cwd: working, timeout: 10 * 60_000 });
      for (const test of names.filter(name => name.startsWith(filter))) assert.ok(output.split('\n').includes(`test ${test} ... ok`), `Required test did not pass: ${name}::${test}`);
    }
  }
  await assertClean(checkout);
  console.log(`[pinned-checkout] PASS: pinned Stripe/Paddle checkout, correlation, price encoding, idempotency, retrieval and timeout tests at ${revision}`);
} catch (error) {
  console.error(error.stack ?? error.message); process.exitCode = 1;
} finally {
  if (active) killGroup(active);
  if (temporary) rmSync(temporary, { recursive: true, force: true });
}
