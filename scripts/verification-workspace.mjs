import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

// Copy only application inputs. No operator environment, database or generated
// assets enter a disposable verification workspace. Include new source files
// for editing-time runs; Cairn checks still require a committed candidate.
export function snapshot(source, prefix) {
  const result = spawnSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], {
    cwd: source, encoding: 'utf8', timeout: 10_000,
  });
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `Cannot inventory the checkout: ${result.stderr}`);
  const scratchRoot = process.env.FOUNDATION_SCRATCH_DIR ?? resolve(source, '../scratchpads');
  mkdirSync(scratchRoot, { recursive: true });
  const destination = mkdtempSync(join(scratchRoot, prefix));
  const roots = ['Cargo.toml', 'Cargo.lock', 'src/', 'cmd/', 'frontend/', 'lang/', 'scripts/', 'tests/', 'README.md', 'handoff.md', '.env.example', 'LICENSE', 'THIRD_PARTY_NOTICES.md'];
  try {
    for (const file of new Set(result.stdout.split('\0').filter(Boolean))) {
      if (!roots.some(root => root.endsWith('/') ? file.startsWith(root) : file === root)) continue;
      if (file.split('/').some(part => (part.startsWith('.env') && part !== '.env.example') || part === 'node_modules' || part === 'target')) continue;
      const target = join(destination, file);
      mkdirSync(dirname(target), { recursive: true });
      cpSync(join(source, file), target);
    }
    return destination;
  } catch (error) {
    rmSync(destination, { recursive: true, force: true });
    throw error;
  }
}
