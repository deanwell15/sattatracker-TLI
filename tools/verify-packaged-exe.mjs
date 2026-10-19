// Tauri 2.10.1 bundle.rs::patch_binary replaces the first UNK bundle token
// with MSI/NSS before packaging and restores the raw EXE afterwards.
// Require exactly that expected patch; every other byte must remain identical.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';

export function verifyPackagedExe(original, packaged, kind) {
  assert(['MSI','NSS'].includes(kind), 'Unexpected bundle type');
  const token = Buffer.from('__TAURI_BUNDLE_TYPE_VAR_UNK');
  const index = original.indexOf(token);
  assert(index >= 0, 'Raw EXE has no Tauri bundle marker');
  assert.equal(original.length, packaged.length, 'Packaged EXE length changed');
  const expected = Buffer.from(original);
  expected.set(Buffer.from(`__TAURI_BUNDLE_TYPE_VAR_${kind}`), index);
  const hash = bytes => createHash('sha256').update(bytes).digest('hex');
  assert.equal(hash(packaged), hash(expected), `Packaged ${kind} EXE differs beyond the expected Tauri bundle marker`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [, , baseline, msi, nsis] = process.argv;
  assert(baseline && msi && nsis, 'Usage: node verify-packaged-exe.mjs raw.exe msi.exe nsis.exe');
  const original = await readFile(baseline);
  verifyPackagedExe(original, await readFile(msi), 'MSI');
  verifyPackagedExe(original, await readFile(nsis), 'NSS');
  console.log('PASS MSI/NSIS EXE payload: only the precise Tauri bundle-type patch differs from raw EXE');
}
