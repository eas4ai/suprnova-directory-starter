// Run a verification command against the actual built Vue renderer on a free port.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const [command, ...args] = process.argv.slice(2);
assert.ok(command, 'Supply the command to run with the local SSR worker.');
const probe = createServer();
await new Promise((resolve, reject) => { probe.once('error', reject); probe.listen(0, '127.0.0.1', resolve); });
const port = probe.address().port;
await new Promise((resolve, reject) => probe.close(error => error ? reject(error) : resolve()));
const env = { ...process.env, SSR_PORT: String(port), SSR_HOST: '127.0.0.1', SSR_URL: `http://127.0.0.1:${port}` };
let worker, child, logs = '', timer, interrupted = false;
async function stop(process) {
  if (!process || process.exitCode !== null || process.signalCode !== null) return;
  process.kill('SIGTERM');
  await Promise.race([new Promise(resolve => process.once('exit', resolve)), delay(3000)]);
  if (process.exitCode === null && process.signalCode === null) process.kill('SIGKILL');
}
const interrupt = () => { interrupted = true; void stop(child); void stop(worker); process.exitCode = 1; };
process.once('SIGTERM', interrupt); process.once('SIGINT', interrupt);
try {
  worker = spawn(process.execPath, ['frontend/bootstrap/ssr/ssr.js'], { env, stdio: ['ignore', 'pipe', 'pipe'] });
  let workerError;
  worker.on('error', error => { workerError = error; });
  for (const stream of [worker.stdout, worker.stderr]) stream.on('data', data => { logs = (logs + data).slice(-24000); });
  let lastProbeError, ready = false;
  for (let attempt = 0; attempt < 150; attempt++) {
    assert.ok(!interrupted, 'SSR verification interrupted.');
    if (workerError) throw workerError;
    if (worker.exitCode !== null || worker.signalCode !== null) throw new Error(`SSR worker exited: ${logs}`);
    try { if ((await fetch(`${env.SSR_URL}/health`, { signal: AbortSignal.timeout(500) })).ok) { ready = true; break; } }
    catch (error) { lastProbeError = error; }
    await delay(100);
  }
  assert.ok(ready, `SSR worker did not start: ${lastProbeError?.message ?? logs}`);
  assert.ok(!interrupted, 'SSR verification interrupted.');
  child = spawn(command, args, { env, stdio: 'inherit' });
  timer = setTimeout(() => { child.kill('SIGKILL'); }, 25 * 60 * 1000);
  const status = await new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', (code, signal) => signal ? reject(new Error(`Verification command stopped by ${signal}`)) : resolve(code));
  });
  assert.equal(status, 0, `Verification command exited ${status}`);
} catch (error) {
  console.error(error.stack ?? error.message);
  if (logs) console.error(logs);
  process.exitCode = 1;
} finally {
  clearTimeout(timer);
  await stop(child); await stop(worker);
}
