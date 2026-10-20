// Build an EXE replacement kit. Never copies item icons or creates installers.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, readdir, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { join, basename } from 'node:path';
import { pathToFileURL } from 'node:url';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function verifyPeX64(bytes) {
  assert(bytes.length >= 64 && bytes.toString('ascii', 0, 2) === 'MZ', 'Missing DOS executable header');
  const pe = bytes.readUInt32LE(0x3c);
  assert(pe >= 64 && pe + 26 <= bytes.length, 'Invalid PE header offset');
  assert(bytes.toString('ascii', pe, pe + 4) === 'PE\0\0', 'Missing PE signature');
  assert.equal(bytes.readUInt16LE(pe + 4), 0x8664, 'Expected Windows x64 machine');
  assert.equal(bytes.readUInt16LE(pe + 24), 0x20b, 'Expected PE32+ optional header');
}
export function iconContract(entries) {
  const icons = entries.filter(e => e.path.startsWith('src-tauri/resources/icons/'))
    .map(e => ({ name: basename(e.path), sha256: e.sha256 })).sort((a,b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0);
  assert.equal(icons.length, 2845, 'Resource edition changed: review EXE replacement compatibility');
  assert.equal(new Set(icons.map(e => e.name)).size, icons.length);
  return { directory: 'icons', protocol: 'deanfinite-icon', iconCount: icons.length,
    // Exact UTF-8, sorted ASCII filenames, LF including the final line.
    iconsSha256: hash(icons.map(e => `${e.name} ${e.sha256}\n`).join('')) };
}
export async function verifyIcons(root, entries) {
  const expected = entries.filter(e => e.path.startsWith('src-tauri/resources/icons/'));
  const names = (await readdir(root)).sort();
  assert.deepEqual(names, expected.map(e => basename(e.path)).sort(), 'Standalone icon filenames differ');
  for (const e of expected) assert.equal(hash(await readFile(join(root, basename(e.path)))), e.sha256, `Icon differs: ${e.path}`);
}
export function verifyFastFiles(names) {
  const allowed = ['Deanfinite.exe','BUILD_INFO.json','RESOURCE_COMPATIBILITY.json','CHECK_COMPATIBILITY.ps1','RUNNING.txt','LICENSE','ASSET_LICENSES.md','SHA256SUMS.txt'];
  assert.deepEqual([...names].sort(), allowed.sort(), 'Fast artifact must contain only the EXE and small metadata/instructions');
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [, , releaseRoot, outputRoot] = process.argv;
  assert(releaseRoot && outputRoot, 'Usage: node prepare-fast-build.mjs releaseRoot outputRoot');
  const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
  assert.deepEqual(config.bundle.resources, {'resources/icons/*.png':'icons/'});
  assert.equal(config.productName, 'Deanfinite');
  assert.equal(config.identifier, 'io.github.deanwell15.deanfinite');
  const manifest = JSON.parse(await readFile('assets/original-assets.manifest.json','utf8'));
  const exe = await readFile(join(releaseRoot, 'Deanfinite.exe'));
  verifyPeX64(exe);
  await verifyIcons(join(releaseRoot, 'icons'), manifest.files);
  const contract = { schemaVersion: 1, productName: config.productName, identifier: config.identifier,
    resourceEdition: manifest.archive_sha256, ...iconContract(manifest.files) };
  await mkdir(outputRoot, {recursive:true});
  await copyFile(join(releaseRoot,'Deanfinite.exe'), join(outputRoot,'Deanfinite.exe'));
  await writeFile(join(outputRoot,'RESOURCE_COMPATIBILITY.json'), JSON.stringify(contract,null,2)+'\n');
  await copyFile('tools/check-standalone-kit.ps1',join(outputRoot,'CHECK_COMPATIBILITY.ps1'));
  for (const name of ['LICENSE','ASSET_LICENSES.md']) await copyFile(name,join(outputRoot,name));
  await writeFile(join(outputRoot,'RUNNING.txt'), `Deanfinite Fast Build: EXE replacement for an EXISTING standalone kit only.
Original pig/coin animations are embedded in EXE. Item icons remain external.
This download has NO icons/, MSI or NSIS. It is NOT self-contained.
Requires Windows x64 and installed Microsoft Edge WebView2 Runtime.

1. Close Deanfinite completely. Never run two copies simultaneously.
2. Back up the entire existing standalone kit, including its EXE and icons/.
3. From this extracted download run PowerShell:
   powershell -NoProfile -File .\\CHECK_COMPATIBILITY.ps1 -KitRoot 'C:\\path\\to\\standalone'
   This checks all 2845 icon hashes. If it fails, use a matching Full Build kit.
4. Replace only Deanfinite.exe in that kit with this new Deanfinite.exe.
5. Keep the existing icons/ directory beside the EXE. Do not delete or move it.
6. Start Deanfinite.exe from the existing kit. Select Automatic (TITrack).
   Confirm season 1601, Cloud cache > 0 and Last successful load in Settings.

BUILD_INFO.json identifies the exact commit/run/profile. SHA256SUMS.txt is not a signature.
Product name, identifier and AppData paths are unchanged. Existing app still requests
administrator privileges and uses its normal AppConfig/LocalAppData/WebView storage.
Original resources remain subject to ASSET_LICENSES.md. No GitHub Release is published.
`);
  const command = (file,args) => execFileSync(file,args,{encoding:'utf8'}).trim();
  const info = { mode: 'fast', repository: process.env.GITHUB_REPOSITORY ?? 'deanwell15/sattatracker-TLI',
    commit: command('git',['rev-parse','HEAD']), ref: process.env.GITHUB_REF ?? null,
    runUrl: process.env.GITHUB_RUN_ID ? `https://github.com/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID}` : null,
    target: 'x86_64-pc-windows-msvc', productName: config.productName, identifier: config.identifier,
    version: config.version, rust: command('rustc',['--version']), node: process.version,
    optimization: { release: true, lto: process.env.CARGO_PROFILE_RELEASE_LTO ?? 'default', codegenUnits: process.env.CARGO_PROFILE_RELEASE_CODEGEN_UNITS ?? 'default' },
    cargoLockSha256: hash(await readFile('src-tauri/Cargo.lock')), npmLockSha256: hash(await readFile('package-lock.json')),
    resourceCompatibility: contract, requiresExistingIcons: true, requiresWebView2: true };
  await writeFile(join(outputRoot,'BUILD_INFO.json'),JSON.stringify(info,null,2)+'\n');
  const sums=[];
  for (const name of (await readdir(outputRoot)).sort()) sums.push(`${hash(await readFile(join(outputRoot,name)))}  ${name}`);
  await writeFile(join(outputRoot,'SHA256SUMS.txt'),sums.join('\n')+'\n');
  verifyFastFiles(await readdir(outputRoot));
  console.log(`PASS Fast EXE PE32+ x64; existing resource layout and ${contract.iconCount} icon hashes; resources NOT copied to artifact`);
}
