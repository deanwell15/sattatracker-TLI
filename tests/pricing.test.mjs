// Bundle TypeScript for Node without opening a development server or downloading tools.
import assert from 'node:assert/strict';
import { build } from 'vite';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
const result = await build({configFile:false, logLevel:'error', build:{ssr:'tests/pricing.test.ts', write:false, minify:false}});
const chunks = result.output.filter(entry => entry.type === 'chunk');
assert.equal(chunks.length,1);
const temp = await mkdtemp(join(import.meta.dirname,'pricing-node-'));
try {
  const path = join(temp,'tests.mjs');
  await writeFile(path,chunks[0].code);
  await import(pathToFileURL(path).href);
} finally { await rm(temp,{recursive:true,force:true}); }
