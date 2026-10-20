import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { verifyPeX64, iconContract, verifyFastFiles } from '../tools/prepare-fast-build.mjs';
const valid = Buffer.alloc(128); valid.write('MZ'); valid.writeUInt32LE(64,0x3c); valid.write('PE\0\0',64); valid.writeUInt16LE(0x8664,68); valid.writeUInt16LE(0x20b,88);
test('Fast artifact requires a Windows PE32+ x64 executable', () => {
  verifyPeX64(valid);
  for (const [offset,value] of [[0,0],[64,0],[68,0x14c],[88,0x10b]]) {
    const wrong = Buffer.from(valid); wrong.writeUInt16LE(value,offset);
    assert.throws(()=>verifyPeX64(wrong));
  }
});
test('Fast artifact rejects installers and copied graphics', () => {
  const files=['Deanfinite.exe','BUILD_INFO.json','RESOURCE_COMPATIBILITY.json','CHECK_COMPATIBILITY.ps1','RUNNING.txt','LICENSE','ASSET_LICENSES.md','SHA256SUMS.txt'];
  verifyFastFiles(files);
  for (const extra of ['icons','setup.exe','app.msi','standalone']) assert.throws(()=>verifyFastFiles([...files,extra]));
});
test('resource contract changes when any icon bytes/name change', async () => {
  const manifest=JSON.parse(await readFile('assets/original-assets.manifest.json','utf8'));
  const expected=iconContract(manifest.files);
  // This edition is the one in the user's previously successful standalone kit.
  const baseline=JSON.parse(await readFile('assets/standalone-resource-contract.json','utf8'));
  assert.deepEqual(expected,baseline);
  const changed=structuredClone(manifest.files);
  changed.find(e=>e.path.startsWith('src-tauri/resources/icons/')).sha256='0'.repeat(64);
  assert.notEqual(iconContract(changed).iconsSha256,expected.iconsSha256);
});
