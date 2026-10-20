<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-[150] flex items-center justify-center pointer-events-auto"
    style="background: rgba(0,0,0,0.75);"
    @click.self="cancel"
  >
    <div
      class="relative text-white rounded-2xl overflow-hidden flex flex-col"
      style="
        width: 460px;
        max-height: 88vh;
        background: linear-gradient(160deg, rgba(14,12,22,0.98) 0%, rgba(10,9,18,0.99) 100%);
        border: 1px solid rgba(255,255,255,0.09);
        box-shadow: 0 20px 60px rgba(0,0,0,0.8), inset 0 1px 0 rgba(255,255,255,0.07);
      "
    >
      <div class="flex items-center justify-between px-5 pt-4 pb-0 shrink-0">
        <span class="text-base font-bold tracking-wide">⚙ Deanfinite Settings</span>
        <button @click="cancel" class="text-white/30 hover:text-white text-lg transition-colors" style="background:none;border:none;cursor:pointer;">✕</button>
      </div>

      <div class="flex gap-1 px-5 pt-3 pb-0 shrink-0" style="border-bottom: 1px solid rgba(255,255,255,0.06);">
        <button
          v-for="tab in TABS" :key="tab.id"
          @click="activeTab = tab.id"
          class="px-4 py-[7px] text-xs font-bold uppercase tracking-widest rounded-t-lg transition-all"
          :style="activeTab === tab.id
            ? 'background: rgba(96,165,250,0.15); color: #60a5fa; border-bottom: 2px solid #60a5fa;'
            : 'color: rgba(255,255,255,0.35); border-bottom: 2px solid transparent;'"
        >{{ tab.label }}</button>
      </div>

      <div class="overflow-y-auto custom-scroll flex-1 px-5 py-4 flex flex-col gap-5">

        <template v-if="activeTab === 'filter'">
          <PricingSettings />

          <section>
            <div class="section-title">Loot Filter</div>
            <div class="flex flex-col gap-3">

              <div class="setting-row">
                <div>
                  <div class="setting-name">Min Drop Value</div>
                  <div class="setting-desc">Hide drops below this value in feed &amp; effects</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.minPrice" min="0" max="20" step="0.5" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-12 text-right">{{ f.minPrice.toFixed(1) }} FE</span>
                </div>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Map Cost (per run)</div>
                  <div class="setting-desc">Auto-added as expense on every new map</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.mapCost" min="0" max="200" step="1" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-12 text-right">{{ f.mapCost }} FE</span>
                </div>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Apply Market Tax (12.5%)</div>
                  <div class="setting-desc">Reduces loot value (Flame Elementium exempt)</div>
                </div>
                <label class="toggle shrink-0">
                  <input type="checkbox" v-model="f.applyTax" />
                  <span class="toggle-track"></span>
                </label>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Ignore Auction / Trade</div>
                  <div class="setting-desc">Auction/trade items won't affect profit</div>
                </div>
                <label class="toggle shrink-0">
                  <input type="checkbox" v-model="f.ignoreAuction" />
                  <span class="toggle-track"></span>
                </label>
              </div>

            </div>
          </section>

          <section>
            <div class="section-title">Loot FX</div>
            <div class="flex flex-col gap-3">

              <div class="setting-row">
                <div>
                  <div class="setting-name">Position X</div>
                  <div class="setting-desc">Horizontal position (% of screen width)</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.fxX" min="0" max="100" step="1" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ f.fxX }}%</span>
                </div>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Position Y</div>
                  <div class="setting-desc">Vertical position (% of screen height)</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.fxY" min="0" max="100" step="1" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ f.fxY }}%</span>
                </div>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Scale</div>
                  <div class="setting-desc">Overall size of the FX notification</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.fxScale" min="0.5" max="2.0" step="0.05" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ f.fxScale.toFixed(2) }}×</span>
                </div>
              </div>

            </div>
          </section>

          <section>
            <div class="section-title">Misc</div>
            <div class="flex flex-col gap-3">

              <div class="setting-row">
                <div>
                  <div class="setting-name">FX Display Duration</div>
                  <div class="setting-desc">How long the drop FX stays on screen</div>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="f.fxDuration" min="1" max="30" step="0.1" class="w-24 accent-blue-500" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ f.fxDuration.toFixed(1) }}s</span>
                </div>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Pause Timer in Hideout</div>
                  <div class="setting-desc">Pauses session timer when entering hub / city</div>
                </div>
                <label class="toggle shrink-0">
                  <input type="checkbox" v-model="f.pauseTimerOnHub" />
                  <span class="toggle-track"></span>
                </label>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Low Stock Warning</div>
                  <div class="setting-desc">Show alert when compass / consumable drops to ≤3</div>
                </div>
                <label class="toggle shrink-0">
                  <input type="checkbox" v-model="f.lowStockEnabled" />
                  <span class="toggle-track"></span>
                </label>
              </div>

              <div class="setting-row">
                <div>
                  <div class="setting-name">Debug Panel</div>
                  <div class="setting-desc">Loot simulator for testing effects</div>
                </div>
                <label class="toggle shrink-0">
                  <input type="checkbox" v-model="f.showDebug" />
                  <span class="toggle-track"></span>
                </label>
              </div>

            </div>
          </section>

        </template>

        <template v-if="activeTab === 'appearance'">

          <section>
            <div class="section-title">Presets</div>
            <div class="flex gap-2 flex-wrap">
              <button
                v-for="(theme, key) in PRESETS" :key="key"
                @click="applyPreset(key)"
                class="px-3 py-1 rounded-lg text-xs font-bold transition-all"
                :style="activePresetId === key
                  ? 'background: rgba(96,165,250,0.25); border: 1px solid #60a5fa; color: #60a5fa;'
                  : 'background: rgba(255,255,255,0.07); border: 1px solid rgba(255,255,255,0.12); color: rgba(255,255,255,0.6);'"
              >{{ theme.name }}</button>
            </div>
          </section>

          <section>
            <div class="section-title">Backgrounds</div>
            <div class="flex flex-col gap-3">
              <ColorRow label="Widget BG"  :hex="c.bgWidget.hex"  :alpha="c.bgWidget.alpha"  @update:hex="c.bgWidget.hex=$event"  @update:alpha="c.bgWidget.alpha=$event"  @update="livePreview" />
              <ColorRow label="Row BG"     :hex="c.bgRow.hex"     :alpha="c.bgRow.alpha"     @update:hex="c.bgRow.hex=$event"     @update:alpha="c.bgRow.alpha=$event"     @update="livePreview" />
              <ColorRow label="Icon BG"    :hex="c.iconBg.hex"    :alpha="c.iconBg.alpha"    @update:hex="c.iconBg.hex=$event"    @update:alpha="c.iconBg.alpha=$event"    @update="livePreview" />
            </div>
          </section>

          <section>
            <div class="section-title">Text Colors</div>
            <div class="flex flex-col gap-3">
              <ColorRow label="Main Text"    :hex="c.textMain.hex"   :alpha="c.textMain.alpha"   :showAlpha="false" @update:hex="c.textMain.hex=$event"   @update:alpha="c.textMain.alpha=$event"   @update="livePreview" />
              <ColorRow label="Price"        :hex="c.textPrice.hex"  :alpha="c.textPrice.alpha"  :showAlpha="false" @update:hex="c.textPrice.hex=$event"  @update:alpha="c.textPrice.alpha=$event"  @update="livePreview" />
              <ColorRow label="Quantity"     :hex="c.textQty.hex"    :alpha="c.textQty.alpha"    :showAlpha="false" @update:hex="c.textQty.hex=$event"    @update:alpha="c.textQty.alpha=$event"    @update="livePreview" />
              <ColorRow label="Header/Title" :hex="c.textHeader.hex" :alpha="c.textHeader.alpha" :showAlpha="false" @update:hex="c.textHeader.hex=$event" @update:alpha="c.textHeader.alpha=$event" @update="livePreview" />
            </div>
          </section>

          <section>
            <div class="section-title">Border</div>
            <ColorRow label="Border Color" :hex="c.borderColor.hex" :alpha="c.borderColor.alpha" @update:hex="c.borderColor.hex=$event" @update:alpha="c.borderColor.alpha=$event" @update="livePreview" />
          </section>

          <section>
            <div class="section-title">Widget Transparency</div>
            <div class="setting-row">
              <div class="setting-name text-sm">Opacity</div>
              <div class="flex items-center gap-2 shrink-0">
                <input type="range" v-model.number="f.widgetOpacity" min="0.1" max="1" step="0.05" class="w-24 accent-blue-500" @input="onOpacityChange" />
                <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ Math.round(f.widgetOpacity * 100) }}%</span>
              </div>
            </div>
          </section>

          <section>
            <div class="section-title">Corner Radius</div>
            <div class="flex flex-col gap-3">
              <div class="setting-row">
                <div class="setting-name text-sm">Widget</div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="r.widget" min="0" max="20" step="1" class="w-24 accent-blue-500" @input="livePreview" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ r.widget }}px</span>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-name text-sm">Row</div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="r.row" min="0" max="12" step="1" class="w-24 accent-blue-500" @input="livePreview" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ r.row }}px</span>
                </div>
              </div>
              <div class="setting-row">
                <div class="setting-name text-sm">Icon</div>
                <div class="flex items-center gap-2 shrink-0">
                  <input type="range" v-model.number="r.icon" min="0" max="8" step="1" class="w-24 accent-blue-500" @input="livePreview" />
                  <span class="text-blue-400 font-bold text-sm w-10 text-right">{{ r.icon }}px</span>
                </div>
              </div>
            </div>
          </section>

        </template>

        <template v-if="activeTab === 'data'">

          <section>
            <div class="section-title">Export Settings</div>
            <div class="setting-desc mb-2">Copy this code to share or back up your settings.</div>
            <textarea
              readonly
              :value="exportCode"
              rows="7"
              class="w-full rounded-lg text-[10px] font-mono resize-none outline-none px-3 py-2 custom-scroll"
              style="background: rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.7);"
            />
            <button
              @click="copyExport"
              class="mt-2 w-full py-[7px] rounded-xl text-xs font-bold transition-all"
              :style="copied
                ? 'background: rgba(74,222,128,0.2); border: 1px solid rgba(74,222,128,0.4); color: #4ade80;'
                : 'background: rgba(96,165,250,0.12); border: 1px solid rgba(96,165,250,0.25); color: #60a5fa;'"
            >{{ copied ? '✓ Copied!' : 'Copy to Clipboard' }}</button>
          </section>

          <section>
            <div class="section-title">Import Settings</div>
            <div class="setting-desc mb-2">Paste a settings code to restore everything.</div>
            <textarea
              v-model="importCode"
              rows="7"
              placeholder="Paste settings code here..."
              class="w-full rounded-lg text-[10px] font-mono resize-none outline-none px-3 py-2 custom-scroll"
              style="background: rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.7);"
            />
            <div v-if="importError" class="mt-1 text-xs text-red-400">{{ importError }}</div>
            <button
              @click="applyImport"
              class="mt-2 w-full py-[7px] rounded-xl text-xs font-bold transition-all"
              style="background: rgba(96,165,250,0.12); border: 1px solid rgba(96,165,250,0.25); color: #60a5fa;"
            >Apply Import</button>
          </section>

          <section>
            <div class="section-title">Reset</div>
            <button
              @click="resetToDefaults"
              class="w-full py-[7px] rounded-xl text-xs font-bold transition-all"
              style="background: rgba(255,80,80,0.1); border: 1px solid rgba(255,80,80,0.25); color: rgba(255,120,120,0.9);"
            >Reset All Settings to Defaults</button>
          </section>

        </template>

      </div>

      <p v-if="saveError" role="alert" class="px-5 pb-2 text-xs text-red-400">{{ saveError }}</p>
      <div class="px-5 pb-4 pt-3 flex gap-2 shrink-0" style="border-top: 1px solid rgba(255,255,255,0.06);">
        <button
          @click="cancel"
          class="flex-1 py-2 rounded-xl font-bold text-xs transition-all"
          style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.5);"
        >Cancel</button>
        <button
          @click="save"
          class="flex-1 py-2 rounded-xl font-bold text-xs transition-all"
          style="background: rgba(96,165,250,0.2); border: 1px solid rgba(96,165,250,0.4); color: #60a5fa;"
        >Save</button>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
import { useThemeStore, PREDEFINED_THEMES, type Theme } from '../stores/themeStore';
import { invoke } from '@tauri-apps/api/core';
import ColorRow from './ColorRow.vue';
import PricingSettings from './PricingSettings.vue';

const props = defineProps<{ isOpen: boolean; showDebugExternal: boolean }>();
const emit  = defineEmits<{
  (e: 'close'): void;
  (e: 'update:showDebug', v: boolean): void;
}>();

const store      = useTrackerStore();
const themeStore = useThemeStore();

const TABS = [
  { id: 'filter'     as const, label: 'Filter' },
  { id: 'appearance' as const, label: 'Appearance' },
  { id: 'data'       as const, label: 'Export / Import' },
];
type TabId = 'filter' | 'appearance' | 'data';
const activeTab = ref<TabId>('filter');
const saveError = ref('');

const PRESETS = PREDEFINED_THEMES;
const activePresetId = ref(themeStore.activeThemeId);

const f = reactive({
  minPrice:        store.minPriceThreshold,
  mapCost:         store.mapCost,
  applyTax:        store.applyTax,
  ignoreAuction:   store.ignoreAuction,
  showDebug:       props.showDebugExternal,
  widgetOpacity:   store.widgetOpacity,
  fxDuration:      store.fxDuration,
  fxX:             store.fxX,
  fxY:             store.fxY,
  fxScale:         store.fxScale,
  pauseTimerOnHub: store.pauseTimerOnHub,
  lowStockEnabled: store.lowStockEnabled,
});

function parseRgba(css: string): { hex: string; alpha: number } {
  const m = css.match(/rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+))?\s*\)/);
  if (m) {
    const [r, g, b] = [+m[1], +m[2], +m[3]];
    const a = m[4] !== undefined ? +m[4] : 1;
    const hex = '#' + [r, g, b].map(v => v.toString(16).padStart(2, '0')).join('');
    return { hex, alpha: a };
  }
  if (css.startsWith('#')) return { hex: css.slice(0, 7), alpha: 1 };
  return { hex: '#111111', alpha: 1 };
}

function buildRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const a = parseFloat(alpha.toFixed(2));
  return a >= 1 ? `rgb(${r},${g},${b})` : `rgba(${r},${g},${b},${a})`;
}

function parseRadius(s: string): number { return parseInt(s) || 0; }

function colorsFromTheme(t: Theme) {
  return {
    bgWidget:    { ...parseRgba(t.colors.bgWidget) },
    bgRow:       { ...parseRgba(t.colors.bgRow) },
    iconBg:      { ...parseRgba(t.colors.iconBg) },
    textMain:    { ...parseRgba(t.colors.textMain) },
    textPrice:   { ...parseRgba(t.colors.textPrice) },
    textQty:     { ...parseRgba(t.colors.textQty) },
    textHeader:  { ...parseRgba(t.colors.textHeader) },
    borderColor: { ...parseRgba(t.colors.borderColor) },
  };
}

function radiiFromTheme(t: Theme) {
  return {
    widget: parseRadius(t.radii.widget),
    row:    parseRadius(t.radii.row),
    icon:   parseRadius(t.radii.icon),
  };
}

const c = reactive(colorsFromTheme(themeStore.currentTheme));
const r = reactive(radiiFromTheme(themeStore.currentTheme));

let snapshotTheme:   Theme  | null = null;
let snapshotOpacity: number | null = null;

watch(() => props.isOpen, (open) => {
  if (!open) return;
  saveError.value = '';
  activeTab.value     = 'filter';
  f.minPrice          = store.minPriceThreshold;
  f.mapCost           = store.mapCost;
  f.applyTax          = store.applyTax;
  f.ignoreAuction     = store.ignoreAuction;
  f.showDebug         = props.showDebugExternal;
  f.widgetOpacity     = store.widgetOpacity;
  f.fxDuration        = store.fxDuration;
  f.fxX               = store.fxX;
  f.fxY               = store.fxY;
  f.fxScale           = store.fxScale;
  f.pauseTimerOnHub   = store.pauseTimerOnHub;
  f.lowStockEnabled   = store.lowStockEnabled;
  activePresetId.value = themeStore.activeThemeId;

  const cols = colorsFromTheme(themeStore.currentTheme);
  const rads  = radiiFromTheme(themeStore.currentTheme);
  Object.keys(cols).forEach(k => { (c as any)[k] = (cols as any)[k]; });
  Object.keys(rads).forEach(k => { (r as any)[k] = (rads as any)[k]; });

  snapshotTheme   = JSON.parse(JSON.stringify(themeStore.currentTheme));
  snapshotOpacity = store.widgetOpacity;

  importCode.value  = '';
  importError.value = '';
  copied.value      = false;
});

function livePreview() {
  const root = document.documentElement;
  root.style.setProperty('--theme-bg-widget',    buildRgba(c.bgWidget.hex,    c.bgWidget.alpha));
  root.style.setProperty('--theme-bg-row',       buildRgba(c.bgRow.hex,       c.bgRow.alpha));
  root.style.setProperty('--theme-icon-bg',      buildRgba(c.iconBg.hex,      c.iconBg.alpha));
  root.style.setProperty('--theme-text-main',    buildRgba(c.textMain.hex,    c.textMain.alpha));
  root.style.setProperty('--theme-text-price',   buildRgba(c.textPrice.hex,   c.textPrice.alpha));
  root.style.setProperty('--theme-text-qty',     buildRgba(c.textQty.hex,     c.textQty.alpha));
  root.style.setProperty('--theme-text-header',  buildRgba(c.textHeader.hex,  c.textHeader.alpha));
  root.style.setProperty('--theme-border-color', buildRgba(c.borderColor.hex, c.borderColor.alpha));
  root.style.setProperty('--theme-radius-widget', `${r.widget}px`);
  root.style.setProperty('--theme-radius-row',    `${r.row}px`);
  root.style.setProperty('--theme-radius-icon',   `${r.icon}px`);
}

function onOpacityChange() {
  store.widgetOpacity = f.widgetOpacity;
}

function applyPreset(key: string) {
  activePresetId.value = key;
  const theme = PRESETS[key];
  if (!theme) return;
  const cols = colorsFromTheme(theme);
  const rads  = radiiFromTheme(theme);
  Object.keys(cols).forEach(k => { (c as any)[k] = (cols as any)[k]; });
  Object.keys(rads).forEach(k => { (r as any)[k] = (rads as any)[k]; });
  livePreview();
}

function buildCurrentTheme(): Theme {
  return {
    id:   activePresetId.value === 'custom' ? 'custom' : activePresetId.value,
    name: PRESETS[activePresetId.value]?.name ?? 'Custom',
    colors: {
      bgWidget:    buildRgba(c.bgWidget.hex,    c.bgWidget.alpha),
      bgRow:       buildRgba(c.bgRow.hex,       c.bgRow.alpha),
      iconBg:      buildRgba(c.iconBg.hex,      c.iconBg.alpha),
      textMain:    buildRgba(c.textMain.hex,    c.textMain.alpha),
      textPrice:   buildRgba(c.textPrice.hex,   c.textPrice.alpha),
      textQty:     buildRgba(c.textQty.hex,     c.textQty.alpha),
      textHeader:  buildRgba(c.textHeader.hex,  c.textHeader.alpha),
      borderColor: buildRgba(c.borderColor.hex, c.borderColor.alpha),
    },
    radii: {
      widget: `${r.widget}px`,
      row:    `${r.row}px`,
      icon:   `${r.icon}px`,
    },
    fonts: { main: themeStore.currentTheme.fonts.main },
  };
}

const importCode  = ref('');
const importError = ref('');
const copied      = ref(false);

const exportCode = computed(() => JSON.stringify({
  v: 1,
  theme: buildCurrentTheme(),
  tracker: {
    minPriceThreshold: f.minPrice,
    mapCost:           f.mapCost,
    applyTax:          f.applyTax,
    ignoreAuction:     f.ignoreAuction,
    widgetOpacity:     f.widgetOpacity,
    dropFxStyle:       store.dropFxStyle,
    extraFxStyle:      store.extraFxStyle,
    widgetPositions:   store.widgetPositions,
  },
}, null, 2));

async function copyExport() {
  try {
    await navigator.clipboard.writeText(exportCode.value);
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch {  }
}

function applyImport() {
  importError.value = '';
  try {
    const data = JSON.parse(importCode.value.trim());
    if (!data || typeof data !== 'object') throw new Error('Invalid format');

    if (data.theme) {
      const theme: Theme = data.theme;
      const cols = colorsFromTheme(theme);
      const rads  = radiiFromTheme(theme);
      Object.keys(cols).forEach(k => { (c as any)[k] = (cols as any)[k]; });
      Object.keys(rads).forEach(k => { (r as any)[k] = (rads as any)[k]; });
      livePreview();
      activePresetId.value = 'custom';
    }
    if (data.tracker) {
      const t = data.tracker;
      if (typeof t.minPriceThreshold === 'number')  f.minPrice      = t.minPriceThreshold;
      if (typeof t.mapCost           === 'number')  f.mapCost       = t.mapCost;
      if (typeof t.applyTax          === 'boolean') f.applyTax      = t.applyTax;
      if (typeof t.ignoreAuction     === 'boolean') f.ignoreAuction = t.ignoreAuction;
      if (typeof t.widgetOpacity     === 'number')  {
        f.widgetOpacity     = t.widgetOpacity;
        store.widgetOpacity = t.widgetOpacity;
      }
      if (t.widgetPositions && typeof t.widgetPositions === 'object') {
        store.widgetPositions = { ...store.widgetPositions, ...t.widgetPositions };
        store.saveConfigToStorage();
      }
    }
    importCode.value = '';
  } catch (e: any) {
    importError.value = 'Invalid code: ' + (e?.message ?? 'parse error');
  }
}

function resetToDefaults() {
  applyPreset('default_v2');
  f.minPrice        = 0.1;
  f.mapCost         = 0;
  f.applyTax        = false;
  f.ignoreAuction   = false;
  f.fxDuration      = 4.0;
  f.fxX             = 50;
  f.fxY             = 50;
  f.fxScale         = 1.0;
  f.pauseTimerOnHub = true;
}

async function save() {
  saveError.value = '';
  store.minPriceThreshold = f.minPrice;
  store.mapCost           = f.mapCost;
  store.applyTax          = f.applyTax;
  store.ignoreAuction     = f.ignoreAuction;
  store.widgetOpacity     = f.widgetOpacity;
  store.fxDuration        = f.fxDuration;
  store.fxX               = f.fxX;
  store.fxY               = f.fxY;
  store.fxScale           = f.fxScale;
  store.pauseTimerOnHub   = f.pauseTimerOnHub;
  store.lowStockEnabled   = f.lowStockEnabled;
  store.saveConfigToStorage();

  try {
    await invoke('save_settings', {
      ignoreAuction: f.ignoreAuction,
      applyTax:      f.applyTax,
    });
  } catch (error) {
    saveError.value = `Deanfinite: не удалось сохранить настройки: ${String(error)}`;
    return;
  }

  const theme = buildCurrentTheme();
  const isPreset = activePresetId.value !== 'custom' && PRESETS[activePresetId.value];
  if (isPreset) {
    themeStore.setTheme(activePresetId.value);
  } else {
    themeStore.customTheme   = theme;
    themeStore.activeThemeId = 'custom';
    themeStore.applyTheme();
    themeStore.saveToStorage();
  }

  emit('update:showDebug', f.showDebug);
  emit('close');
}

function cancel() {
  store.widgetOpacity = store.widgetOpacity;
  if (snapshotOpacity !== null) store.widgetOpacity = snapshotOpacity;

  if (snapshotTheme) {
    const isKnown = !!PRESETS[snapshotTheme.id];
    if (isKnown) {
      themeStore.setTheme(snapshotTheme.id);
    } else {
      themeStore.customTheme   = snapshotTheme;
      themeStore.activeThemeId = 'custom';
      themeStore.applyTheme();
    }
  }
  emit('close');
}
</script>
