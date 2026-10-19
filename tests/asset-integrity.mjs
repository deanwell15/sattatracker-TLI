import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { basename } from 'node:path';

const manifest = JSON.parse(await readFile('assets/original-assets.manifest.json', 'utf8'));
assert.equal(manifest.archive_sha256, '83e55bb0e7d193e43a66fa2304f5eff867360fc10b51ee29b45d74ac708669ab');
assert.equal(manifest.files.length, 3088);
assert.equal(manifest.file_count, manifest.files.length);
const byPath = new Map();
let total = 0;
for (const entry of manifest.files) {
  assert(!byPath.has(entry.path), `Duplicate manifest path: ${entry.path}`);
  assert(/^(?:src-tauri\/resources\/icons\/(?:\d{1,12}|fe)|src\/assets\/animations\/pig\/pig_\d+|src\/assets\/animations\/coin\/coin_\d+)\.png$/.test(entry.path));
  const bytes = await readFile(entry.path);
  assert.equal(bytes.length, entry.bytes, entry.path);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, entry.path);
  const png = bytes.subarray(0, 8).equals(Buffer.from([137,80,78,71,13,10,26,10]));
  const webp = bytes.toString('ascii', 0, 4) === 'RIFF' && bytes.toString('ascii', 8, 12) === 'WEBP';
  assert(png || webp, `Unexpected actual format: ${entry.path}`);
  assert.equal(entry.mime, png ? 'image/png' : 'image/webp');
  assert(entry.width > 0 && entry.height > 0 && entry.width <= 4096 && entry.height <= 4096);
  total += bytes.length;
  byPath.set(entry.path, entry);
}
assert.equal(total, manifest.total_bytes);
for (let tier = 0; tier < 4; tier++) {
  const frames = Array.from({length:30}, (_,i) => byPath.get(`src/assets/animations/pig/pig_${tier*30+i}.png`));
  assert(frames.every(Boolean), `Missing pig tier ${tier}`);
  assert.equal(new Set(frames.map(x => x.pixels_sha256)).size, 30, `Pig tier ${tier} contains identical frames`);
}
const coins = Array.from({length:122}, (_,i) => byPath.get(`src/assets/animations/coin/coin_${i}.png`));
assert(coins.every(Boolean), 'Coin sequence incomplete');
assert.equal(new Set(coins.map(x => x.pixels_sha256)).size, 122);
const icons = manifest.files.filter(x => x.path.startsWith('src-tauri/resources/icons/'));
assert.equal(icons.length, 2845);
assert.notEqual(byPath.get('src-tauri/resources/icons/100300.png').pixels_sha256, byPath.get('src-tauri/resources/icons/100200.png').pixels_sha256);
const registry = JSON.parse(await readFile('src/assets/original-icons.json', 'utf8'));
assert.deepEqual([...registry.ids].sort(), icons.map(x => basename(x.path, '.png')).sort());
for (const dir of ['src-tauri/resources/icons','src/assets/animations/pig','src/assets/animations/coin']) {
  const listed = (await readdir(dir)).map(name => `${dir}/${name}`).sort();
  assert.deepEqual(listed, manifest.files.filter(x => x.path.startsWith(`${dir}/`)).map(x=>x.path).sort());
}
console.log(`PASS original bytes/MIME/registry: ${manifest.files.length} images, ${total} bytes; pig 4×30 + coin 122 distinct frames`);
