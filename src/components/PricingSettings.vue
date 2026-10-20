<template>
  <section class="flex flex-col gap-3">
    <div class="section-title">Market prices</div>
    <label class="setting-name" for="price-source">Source</label>
    <select id="price-source" :value="store.pricing?.source ?? 'local_json'" @change="changeSource" class="bg-slate-800 text-white rounded p-2" :disabled="busy">
      <option value="automatic">Automatic (TITrack)</option>
      <option value="local_json">Local JSON</option>
    </select>
    <p class="setting-desc">Season: {{ store.pricing?.season_id ?? 1601 }} · {{ store.pricing?.available_count ?? 1 }} priced IDs (including FE)</p>
    <p class="setting-desc">Cloud cache: {{ store.pricing?.cloud_count ?? 0 }} IDs · Local table: {{ store.pricing?.local_count ?? 0 }} IDs</p>
    <p v-if="store.pricing?.last_sync" class="setting-desc">Last sync: {{ store.pricing.last_sync.received }} received · {{ store.pricing.last_sync.accepted }} accepted · {{ store.pricing.last_sync.skipped }} skipped · {{ store.pricing.last_sync.low_observation_rows }} with low observation counts (accepted)</p>
    <p class="setting-desc">Connection: {{ store.pricing?.connection ?? 'initializing' }} · refresh: 5 minutes</p>
    <p class="setting-desc">Last successful load: {{ store.pricing?.last_success_at ?? 'never' }}</p>
    <p v-if="store.pricing?.next_retry_at" class="setting-desc">Next request: {{ store.pricing.next_retry_at }}</p>
    <p v-if="store.pricing?.error" class="text-xs text-amber-300" role="status">{{ store.pricing.error }}</p>
    <label class="setting-name" for="price-json">Import local prices (.json, max 4 MiB)</label>
    <input id="price-json" type="file" accept=".json,application/json" @change="importPrices" :disabled="busy" class="text-xs" />
    <p class="setting-desc">Import replaces the local table for this season. Unknown prices remain unpriced; quantities are always counted. Cached estimates older than 24 hours are marked stale.</p>
    <p v-if="error" class="text-xs text-red-300" role="alert">{{ error }}</p>
    <p v-if="notice" class="text-xs text-green-300" role="status">{{ notice }}</p>
  </section>
</template>
<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useTrackerStore } from '../stores/trackerStore';
import type { PricingSnapshot } from '../utils/prices';
const store = useTrackerStore();
const busy = ref(false), error = ref(''), notice = ref('');
async function changeSource(event: Event) {
  const select = event.target as HTMLSelectElement;
  busy.value = true; error.value = ''; notice.value = '';
  try { store.applyPricing(await invoke<PricingSnapshot>('set_price_source', { source: select.value })); }
  catch (e) { error.value = String(e); select.value = store.pricing?.source ?? 'local_json'; }
  finally { busy.value = false; }
}
async function importPrices(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  busy.value = true; error.value = ''; notice.value = '';
  try {
    if (file.size > 4 * 1024 * 1024) throw new Error('Price JSON exceeds 4 MiB');
    const snapshot = await invoke<PricingSnapshot>('import_price_json', { json: await file.text() });
    store.applyPricing(snapshot);
    notice.value = 'Local prices imported. Existing quantities preserved; no effects replayed.';
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; input.value = ''; }
}
</script>
