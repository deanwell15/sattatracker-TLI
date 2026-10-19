import { test } from 'node:test';
import assert from 'node:assert/strict';
import { verifyPackagedExe } from '../tools/verify-packaged-exe.mjs';
const original = Buffer.from('prefix\0__TAURI_BUNDLE_TYPE_VAR_UNK\0graphics-payload\0__TAURI_BUNDLE_TYPE_VAR_UNK');
function patched(kind) {
  const bytes = Buffer.from(original);
  bytes.set(Buffer.from(`__TAURI_BUNDLE_TYPE_VAR_${kind}`), original.indexOf('__TAURI_BUNDLE_TYPE_VAR_UNK'));
  return bytes;
}
test('accepts only the first Tauri token patch for each installer', () => {
  verifyPackagedExe(original, patched('MSI'), 'MSI');
  verifyPackagedExe(original, patched('NSS'), 'NSS');
});
test('rejects a changed embedded payload byte', () => {
  const changed = patched('MSI'); changed[changed.indexOf('graphics-payload')] ^= 1;
  assert.throws(() => verifyPackagedExe(original, changed, 'MSI'));
});
test('rejects wrong type, unpatched executable and length changes', () => {
  assert.throws(() => verifyPackagedExe(original, patched('NSS'), 'MSI'));
  assert.throws(() => verifyPackagedExe(original, original, 'MSI'));
  assert.throws(() => verifyPackagedExe(original, Buffer.concat([patched('MSI'),Buffer.from('x')]), 'MSI'));
});
