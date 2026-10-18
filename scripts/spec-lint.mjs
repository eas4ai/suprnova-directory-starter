// Use the parser and linter shipped with the installed Cairn checkout.
import { execFileSync, spawnSync } from 'node:child_process';
import { realpathSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
try {
  const executable = execFileSync('which', ['cairn'], { encoding: 'utf8' }).trim();
  const root = resolve(dirname(realpathSync(executable)), '..');
  const result = spawnSync(process.execPath, [resolve(root, 'scripts/spec-lint.mjs'), ...process.argv.slice(2)], { stdio: 'inherit' });
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
} catch (error) {
  console.error(`Spec lint requires the installed Cairn checkout: ${error.message}`);
  process.exitCode = 1;
}
