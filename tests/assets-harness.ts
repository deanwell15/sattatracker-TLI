import { createApp, h, nextTick, ref } from 'vue';
import { createPinia } from 'pinia';
import { mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import { emit } from '@tauri-apps/api/event';
import AssetImage from '../src/components/AssetImage.vue';
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
mockIPC(cmd => {
  commands.push(cmd);
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
  await emit('game_event', { type: 'LootDrop', item_id: '100300', item_name: 'Offline fixture', qty: 2, price: 10 });
  await nextTick();
  const text = document.querySelector('#app')!.textContent!;
  assert(text.includes('Offline fixture') && text.includes('x2') && text.includes('[20.0]'), 'Item text or values lost');
  assert(text.includes('Session: 20.0 FE'), 'Existing event/statistics path failed');

  const probe = ref('/__missing_asset__.png');
  const probes = createApp({ render: () => h('div', [
    h(AssetImage, { src: probe.value, alt: 'probe' }),
    h(AssetImage, { src: getAnim('coin/coin_121'), alt: 'coin' }),
    h(AssetImage, { src: getAnim('pig_119'), alt: 'effect' }),
    h(AssetImage, { src: getIcon('../../unknown-id'), alt: 'unknown item' }),
  ]) });
  probes.mount('#probes');
  const terminal = scenario === 'missing' || scenario === 'corrupt';
  await eventually(() => terminal
    ? document.querySelectorAll('[data-asset-fallback]').length >= 4
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
    await eventually(() => !!document.querySelector<HTMLImageElement>('#probes img[alt="probe"]')?.src.includes('/coin.svg'), 'Image source did not recover');
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
