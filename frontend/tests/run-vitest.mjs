import { spawn } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const cwd = path.resolve(__dirname, '..');
const loader = path.resolve(cwd, 'tests/loader.mjs');
const patterns = process.argv.slice(2);

function collectSpecs(dir) {
  const entries = readdirSync(dir);
  const files = [];
  for (const entry of entries) {
    const full = path.join(dir, entry);
    const stats = statSync(full);
    if (stats.isDirectory()) {
      files.push(...collectSpecs(full));
    } else if (full.endsWith('.spec.ts')) {
      files.push(full);
    }
  }
  return files;
}

const specs = patterns.length ? patterns : collectSpecs(path.resolve(cwd, 'tests'));

const child = spawn(
  process.execPath,
  ['--loader', loader, '--test', ...specs],
  {
    stdio: 'inherit',
    cwd,
    env: {
      ...process.env,
      NODE_PATH: [path.resolve(cwd, 'tests/vendor'), path.resolve(cwd, 'tests/mocks'), process.env.NODE_PATH || '']
        .filter(Boolean)
        .join(':'),
    },
  },
);

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
