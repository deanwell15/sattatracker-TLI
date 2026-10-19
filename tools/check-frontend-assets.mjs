import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { join } from 'node:path';
const directory = process.argv[2] || 'dist';
const manifest = JSON.parse(await readFile('assets/original-assets.manifest.json', 'utf8'));
const expected = manifest.files.filter(x => x.path.startsWith('src/assets/animations/'));
const hashes = new Set();
for (const file of await readdir(join(directory, 'assets'))) {
  if (/^(pig|coin)_\d+-.*\.png$/.test(file)) {
    hashes.add(createHash('sha256').update(await readFile(join(directory,'assets',file))).digest('hex'));
  }
}
for (const frame of expected) assert(hashes.has(frame.sha256), `Animation absent/re-encoded in frontend: ${frame.path}`);
const html = await readFile(join(directory,'index.html'),'utf8');
assert(!/https?:\/\/(?!localhost)/.test(html), 'External frontend resource');
console.log(`PASS production frontend contains all ${expected.length} original animation files (120 used pig + spare + 122 coin)`);
