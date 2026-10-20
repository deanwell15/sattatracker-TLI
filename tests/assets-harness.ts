import { createApp, h, nextTick, ref } from 'vue';
import { createPinia } from 'pinia';
import { mockIPC, mockWindows, mockConvertFileSrc } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import AssetImage from '../src/components/AssetImage.vue';
import PiggyWidget from '../src/components/PiggyWidget.vue';
import ItemsHistory from '../src/components/ItemsHistory.vue';
import PricingSettings from '../src/components/PricingSettings.vue';
import type { PricingSnapshot, PriceQuote } from '../src/utils/prices';
import { getIcon, getAnim } from '../src/utils/assets';
import { useTrackerStore } from '../src/stores/trackerStore';
import '../src/style.css';

const scenario = new URLSearchParams(location.search).get('scenario') || 'clean';
const commands: string[] = [];
const errors: string[] = [];
window.addEventListener('error', e => { if (e.message) errors.push(e.message); });
window.addEventListener('unhandledrejection', e => errors.push(String(e.reason)));
window.addEventListener('securitypolicyviolation', e => errors.push(`External resource blocked: ${e.blockedURI}`));
mockWindows(scenario === 'widget' ? 'widget_lootFeedSession' : 'main');
mockConvertFileSrc('windows');
const internals = (window as unknown as { __TAURI_INTERNALS__: { convertFileSrc: (file: string, protocol: string) => string } }).__TAURI_INTERNALS__;
internals.convertFileSrc = (file, protocol) => {
  if (protocol !== 'deanfinite-icon' || !/^(?:[0-9]{1,12}|fe)\.png$/.test(file)) throw new Error('Unsafe icon protocol request');
  return `${location.origin}/__icons/${file}`;
};
let mockedPricing: PricingSnapshot | null = null;
mockIPC((cmd, payload) => {
  commands.push(cmd);
  if (cmd === 'get_pricing_snapshot') return mockedPricing;
  if (cmd === 'set_price_source') return { ...mockedPricing!, revision: mockedPricing!.revision + 1, source: 'automatic', connection: 'offline_cache', error: 'TITrack HTTP 503' };
  if (cmd === 'import_price_json') {
    if ((payload as {json:string}).json !== '{"fixture":true}') throw new Error('Invalid price JSON fixture');
    return { ...mockedPricing!, revision: mockedPricing!.revision + 2 };
  }
  if (cmd === 'get_cursor_position') return [0, 0];
  if (cmd === 'plugin:app|version') return '1.2.11';
  return null;
}, { shouldMockEvents: true });

const assert = (condition: unknown, message: string) => { if (!condition) throw new Error(message); };
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
async function eventually(check: () => boolean, message: string) {
  for (let i = 0; i < 150; i++) { if (check()) return; await delay(20); }
  throw new Error(message);
}

try {
  const seen = localStorage.getItem('asset-test-restart');
  assert(scenario === 'restart' ? seen === 'saved' : seen === null, 'Unexpected profile state');
  document.cookie = `asset-test=${scenario}; path=/; SameSite=Strict`;
  const { default: App } = await import('../src/App.vue');
  const pinia = createPinia();
  createApp(App).use(pinia).mount('#app');
  const store = useTrackerStore(pinia);
  // Deliberately never emit LoadingDone: core widgets must render independently.
  await eventually(() => document.querySelector('#app')!.textContent!.includes('Session: 0.0 FE'), 'Loot widget not rendered');
  await eventually(() => store.fxScale === (scenario === 'restart' ? 1.25 : 1), 'Frontend settings did not survive restart');
  assert([...document.querySelectorAll('#app div')].some(el => el.textContent?.trim() === 'Session: 0.0 FE' && el.getBoundingClientRect().height > 0), 'Loot widget is hidden');
  if (scenario !== 'widget') {
    assert(document.querySelector('#app')!.textContent!.includes('Profit / Hr'), 'Statistics widget missing');
  } else {
    assert(!document.querySelector('#app')!.textContent!.includes('Profit / Hr'), 'Detached route rendered main widgets');
  }
  // Real App event handler/store/row rendering; tax is bypassed by existing FE ID.
  await emit('game_event', { type: 'LootDrop', item_id: '100300', item_name: 'Offline fixture', qty: 2, price: 1 });
  await nextTick();
  const text = document.querySelector('#app')!.textContent!;
  assert(text.includes('Offline fixture') && text.includes('x2') && text.includes('[2.0]'), 'Item text or values lost');
  assert(text.includes('Session: 2.0 FE'), 'Existing event/statistics path failed');

  // Real Rust event shape -> App handler -> Pinia -> LootFeed / history / settings.
  store.minPriceThreshold = 100;
  const unpriced: PriceQuote = { value: null, status: 'unknown', source: null, updated_at: null, received_at: null };
  const priced: PriceQuote = { value: 12.5, status: 'known', source: 'test_fixture', updated_at: '2026-10-20T08:00:00Z', received_at: '2026-10-20T08:00:00Z' };
  await emit('game_event', { type: 'LootDrop', item_id: '123456', item_name: 'Unpriced fixture', qty: 3, price: 0, valuation: unpriced, season_id: 1601 });
  await emit('game_event', { type: 'LootDrop', item_id: '123456', item_name: 'Unpriced fixture', qty: 1, price: 0, valuation: unpriced, season_id: 1601 });
  await nextTick();
  assert(store.sessionLootCount === 6 && store.sessionDrops.find(i => i.item_id === '123456')?.qty === 4 && store.currentMapDrops.find(i => i.item_id === '123456')?.qty === 4, 'Unpriced loot lost or double-counted');
  assert(!document.querySelector('#app')!.textContent!.includes('Unpriced fixture'), 'Feed ignored positive display filter');
  store.minPriceThreshold = 0;
  await nextTick();
  assert(document.querySelector('#app')!.textContent!.includes('Unpriced fixture') && document.querySelector('#app')!.textContent!.includes('[—]'), 'Threshold did not reveal retained unpriced loot');
  const historyRoot = document.createElement('div'); document.body.append(historyRoot);
  const historyApp = createApp(ItemsHistory, {isOpen: true}).use(pinia); historyApp.mount(historyRoot);
  assert(historyRoot.textContent!.includes('Unpriced fixture') && historyRoot.textContent!.includes('x4 @ —'), 'History lost unpriced quantity/status');
  historyApp.unmount(); historyRoot.remove();
  const lastDrop = store.latestDrop;
  mockedPricing = {revision: 10, season_id: 1601, source: 'local_json', connection: 'offline_local', last_success_at: null, error: null, available_count: 2, refresh_seconds: 300, titrack_enabled: false,
    prices: {'123456': priced, '100300': {...priced, value: 1, source: 'base_currency', updated_at: null}}};
  await emit('price_snapshot', mockedPricing); await nextTick();
  assert(store.totalProfit === 52 && store.currentMapProfit === 52 && store.sessionLootCount === 6, 'Price snapshot did not revalue existing quantities');
  assert(store.latestDrop === lastDrop, 'Price snapshot replayed drop/effects');
  assert(document.querySelector('#app')!.textContent!.includes('[50.0]'), 'Updated estimate did not render');
  await emit('game_event', {type:'MapStart', level_type:3, level_id:456}); await nextTick();
  assert(store.currentMapDrops.length === 0 && store.sessionLootCount === 6, 'Map change lost session history');
  const settingsRoot = document.createElement('div'); document.body.append(settingsRoot);
  const settingsApp = createApp(PricingSettings).use(pinia); settingsApp.mount(settingsRoot);
  const select = settingsRoot.querySelector('select')!; select.value = 'automatic'; select.dispatchEvent(new Event('change'));
  await eventually(() => settingsRoot.textContent!.includes('TITrack HTTP 503'), 'Unavailable TITrack status not shown');
  const input = settingsRoot.querySelector<HTMLInputElement>('input[type=file]')!;
  for (const [json, message] of [['{bad', 'Invalid price JSON fixture'], ['{"fixture":true}', 'Local prices imported.']] as const) {
    const files = new DataTransfer(); files.items.add(new File([json], 'prices.json', {type:'application/json'})); input.files = files.files; input.dispatchEvent(new Event('change'));
    await eventually(() => settingsRoot.textContent!.includes(message), `Import UI failed: ${message}`);
    assert(store.sessionLootCount === 6 && store.totalProfit === 52, 'Import changed quantity or destroyed prior valuation');
  }
  settingsApp.unmount(); settingsRoot.remove();

  const probe = ref('/__missing_asset__.png');
  const probes = createApp({ render: () => h('div', [
    h(AssetImage, { src: probe.value, alt: 'probe' }),
    h(AssetImage, { src: getAnim('coin/coin_121'), alt: 'coin' }),
    h(AssetImage, { src: getAnim('pig_119'), alt: 'effect' }),
    h(AssetImage, { src: getIcon('../../unknown-id'), alt: 'unknown item' }),
    h(AssetImage, { src: getIcon('100300'), alt: 'FE icon' }),
    h(AssetImage, { src: getIcon('100200'), alt: 'other icon' }),
  ]) });
  probes.mount('#probes');
  const terminal = scenario === 'missing' || scenario === 'corrupt';
  await eventually(() => terminal
    ? document.querySelectorAll('[data-asset-fallback]').length >= 6
    : [...document.querySelectorAll<HTMLImageElement>('#probes img')].every(img => img.complete && img.naturalWidth > 0),
  'Missing image fallback failed');
  if (terminal) {
    assert(document.querySelector('#app [data-asset-fallback]'), 'Loot image failure has no visible fallback');
  } else {
    probe.value = 'data:image/png;base64,bm90IGEgcG5n';
    await nextTick();
    await eventually(() => !!document.querySelector<HTMLImageElement>('#probes img[alt="probe"]')?.src.includes('/item.svg'), 'Corrupt image fallback failed');
    probe.value = getAnim('coin/coin_0');
    await nextTick();
    await eventually(() => !!document.querySelector<HTMLImageElement>('#probes img[alt="probe"]')?.src.includes('/coin_0.png'), 'Image source did not recover');
  }
  if (!terminal) {
    assert(getIcon('100300') !== getIcon('100200'), 'Individual icon mapping lost');
    assert(getIcon('../100300') === getIcon('unknown-item'), 'Invalid ID did not fall back');
    for (const [prefix, count] of [['pig_', 120], ['coin/coin_', 122]] as const) {
      const urls = Array.from({length:count}, (_,i)=>getAnim(`${prefix}${i}`));
      assert(new Set(urls).size === count, 'Animation frames collapsed to one image');
      for (let i=0; i<count; i++) assert(urls[i].includes(`${prefix.split('/').pop()}${i}.png`), 'Frame order mismatch');
    }
    const animation = document.createElement('div');
    animation.style.cssText = 'width:240px;height:180px';
    document.body.append(animation);
    const animatedApp = createApp(PiggyWidget).use(pinia);
    animatedApp.mount(animation);
    await eventually(() => !!animation.querySelector<HTMLImageElement>('img')?.complete, 'Pig frame did not decode');
    const initial = animation.querySelector('img')!.getAttribute('src');
    await eventually(() => animation.querySelector('img')?.getAttribute('src') !== initial, 'Pig animation did not advance');
    animation.firstElementChild!.dispatchEvent(new MouseEvent('click', {bubbles:true}));
    await eventually(() => !!animation.querySelector('img')?.getAttribute('src')?.includes('/coin_'), 'Click did not trigger original coin frames');
    const coin = animation.querySelector('img')!.getAttribute('src');
    await eventually(() => animation.querySelector('img')?.getAttribute('src') !== coin, 'Coin sequence did not advance');
    animatedApp.unmount();
    animation.remove();
  }
  await delay(100);
  assert(!commands.some(cmd => /check_assets_update|download_assets|get_assets_path/.test(cmd)), 'Legacy asset IPC invoked');
  assert(!performance.getEntriesByType('resource').some(entry => /^https?:/.test(entry.name) && new URL(entry.name).origin !== location.origin), 'External resource requested');
  assert(errors.length === 0, errors.join('\n'));
  store.fxScale = 1.25;
  store.saveConfigToStorage();
  localStorage.setItem('asset-test-restart', 'saved');
  document.querySelector('#result')!.textContent = `PASS ${scenario}`;
} catch (error) {
  document.querySelector('#result')!.textContent = `FAIL ${scenario}: ${String(error)}\n${errors.join('\n')}\n${document.querySelector('#probes')!.innerHTML}`;
}
