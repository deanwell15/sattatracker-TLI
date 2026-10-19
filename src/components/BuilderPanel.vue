<template>
  <Transition name="builder-slide">
    <div
      v-if="isOpen"
      ref="panelRef"
      data-interactive
      class="fixed pointer-events-auto select-none z-[200]"
      :style="{ left: pos.x + 'px', top: pos.y + 'px' }"
    >
      <div
        class="overflow-hidden"
        style="
          width: 480px;
          background: linear-gradient(160deg, rgba(14,12,22,0.99) 0%, rgba(10,9,18,0.995) 100%);
          border: 1px solid rgba(255,255,255,0.09);
          border-radius: 18px;
          box-shadow: 0 24px 80px rgba(0,0,0,0.85), inset 0 1px 0 rgba(255,255,255,0.06);
        "
      >
        <div
          class="flex items-center justify-between px-5 py-3 cursor-move"
          style="border-bottom: 1px solid rgba(255,255,255,0.06);"
          @mousedown="startDrag"
        >
          <div class="flex items-center gap-3">
            <div class="w-[7px] h-[7px] rounded-full" style="background:#4ade80; box-shadow:0 0 6px #4ade80;"></div>
            <div>
              <div class="text-[13px] font-black tracking-wide" style="color: rgba(255,255,255,0.85);">Deanfinite</div>
              <div class="text-[9px] font-mono" style="color: rgba(255,255,255,0.22);">v{{ VERSION }} · Setup Panel</div>
            </div>
          </div>

          <div class="flex items-center gap-2" @mousedown.stop>
            <button
              @click="store.isEditMode = !store.isEditMode"
              class="flex items-center gap-[6px] px-3 py-[4px] rounded-lg text-[11px] font-semibold transition-all duration-150"
              :style="{
                background: store.isEditMode ? 'rgba(96,165,250,0.2)' : 'rgba(255,255,255,0.05)',
                border: store.isEditMode ? '1px solid rgba(96,165,250,0.5)' : '1px solid rgba(255,255,255,0.1)',
                color: store.isEditMode ? '#60a5fa' : 'rgba(255,255,255,0.4)',
              }"
            >
              <span>{{ store.isEditMode ? '✓' : '⊹' }}</span>
              <span>Edit Layout</span>
            </button>
            <span class="text-[9px] font-mono" style="color:rgba(255,255,255,0.15);">F8</span>
            <button
              @click="$emit('close')"
              class="text-[15px] ml-1"
              style="color:rgba(255,255,255,0.25); background:none; border:none; cursor:pointer; line-height:1; padding:2px 5px; border-radius:4px;"
              onmouseover="this.style.color='rgba(255,255,255,0.7)'"
              onmouseout="this.style.color='rgba(255,255,255,0.25)'"
            >✕</button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-0" style="border-bottom: 1px solid rgba(255,255,255,0.06);">

          <div class="px-4 py-4" style="border-right: 1px solid rgba(255,255,255,0.06);">
            <div class="text-[9px] font-bold uppercase tracking-[3px] mb-[10px]" style="color: rgba(255,255,255,0.22);">Active Modules</div>
            <div class="flex flex-col gap-[2px]">
              <label
                v-for="(label, key) in widgetToggles"
                :key="key"
                class="flex items-center gap-[8px] cursor-pointer py-[4px] px-[6px] rounded-lg transition-colors duration-150 hover:bg-white/5"
              >
                <div
                  class="w-[13px] h-[13px] rounded-[3px] flex items-center justify-center shrink-0 transition-all duration-150"
                  :style="{
                    background: isWidgetVisible(key) ? 'rgba(74,222,128,0.2)' : 'rgba(255,255,255,0.04)',
                    border: isWidgetVisible(key) ? '1px solid rgba(74,222,128,0.45)' : '1px solid rgba(255,255,255,0.1)',
                  }"
                >
                  <span v-if="isWidgetVisible(key)" class="text-[8px]" style="color:#4ade80;">✓</span>
                </div>
                <input type="checkbox" :checked="isWidgetVisible(key)" class="hidden" @change="toggleWidget(key)" />
                <span class="text-[12px]" style="color: rgba(255,255,255,0.6);">{{ label }}</span>
              </label>
            </div>
          </div>

          <div class="px-4 py-4 flex flex-col gap-4">
            <div>
              <div class="text-[9px] font-bold uppercase tracking-[3px] mb-3" style="color: rgba(255,255,255,0.22);">Visual Effects</div>
              <div class="flex flex-col gap-[8px]">
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[11px] font-semibold" style="color: rgba(255,255,255,0.5);">Drop FX</span>
                  <select v-model="store.dropFxStyle" class="setting-select" style="min-width:120px;font-size:11px;padding:3px 8px;" @change="store.saveConfigToStorage()">
                    <option value="modern">Modern (card)</option>
                    <option value="classic">Classic (glow)</option>
                    <option value="none">Off</option>
                  </select>
                </div>
                <div class="flex items-center justify-between gap-2">
                  <span class="text-[11px] font-semibold" style="color: rgba(255,255,255,0.5);">Extra FX</span>
                  <select v-model="store.extraFxStyle" class="setting-select" style="min-width:130px;font-size:11px;padding:3px 8px;" @change="store.saveConfigToStorage()">
                    <option value="piggy">Piggy Bank</option>
                    <option value="casino">Casino</option>
                    <option value="web">Web</option>
                    <option value="chain">Chain</option>
                    <option value="orbital">Orbital</option>
                    <option value="bubbles">Bubbles</option>
                    <option value="swarm">Swarm</option>
                    <option value="rings">Energy Rings</option>
                    <option value="slash">Slash (≥30 FE)</option>
                    <option value="shockwave">Shockwave (≥50 FE)</option>
                    <option value="confetti">Confetti</option>
                    <option value="lightning">Lightning (≥80 FE)</option>
                    <option value="none">Off</option>
                  </select>
                </div>
              </div>
            </div>

            <div style="height:1px; background: linear-gradient(90deg, transparent, rgba(255,255,255,0.07) 50%, transparent);"></div>

            <div class="flex flex-col gap-[6px]">
              <button @click="$emit('open-history')" class="builder-action-btn" style="--btn-color:#fbbf24;--btn-bg:rgba(251,191,36,0.08);--btn-border:rgba(251,191,36,0.2);">
                <span>🎒</span><span>Inventory / Items</span>
              </button>
              <button @click="$emit('open-settings')" class="builder-action-btn" style="--btn-color:rgba(255,255,255,0.7);--btn-bg:rgba(255,255,255,0.05);--btn-border:rgba(255,255,255,0.1);">
                <span>⚙</span><span>Settings</span>
              </button>
              <button @click="store.resetSession()" class="builder-action-btn" style="--btn-color:#f87171;--btn-bg:rgba(248,113,113,0.08);--btn-border:rgba(248,113,113,0.2);">
                <span>↺</span><span>Reset Session</span>
              </button>
            </div>
          </div>
        </div>

      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
import { getVersion } from '@tauri-apps/api/app';

defineProps<{ isOpen: boolean }>();
defineEmits(['close', 'open-settings', 'open-history']);

const VERSION = ref('1.0.14');

onMounted(async () => {
  try {
    VERSION.value = await getVersion();
  } catch (e) {
    console.error('Failed to load version:', e);
  }
});

const store   = useTrackerStore();
const panelRef = ref<HTMLElement | null>(null);

const pos = ref({ x: Math.max(0, window.innerWidth / 2 - 240), y: Math.max(0, window.innerHeight / 2 - 200) });

let dragging = false, dragX0 = 0, dragY0 = 0, posX0 = 0, posY0 = 0;

function startDrag(e: MouseEvent) {
  dragging = true;
  dragX0 = e.clientX; dragY0 = e.clientY;
  posX0 = pos.value.x; posY0 = pos.value.y;
  window.addEventListener('mousemove', onDrag);
  window.addEventListener('mouseup', stopDrag);
}
function onDrag(e: MouseEvent) {
  if (!dragging) return;
  pos.value.x = posX0 + (e.clientX - dragX0);
  pos.value.y = Math.max(0, posY0 + (e.clientY - dragY0));
}
function stopDrag() {
  dragging = false;
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
}
onUnmounted(() => {
  window.removeEventListener('mousemove', onDrag);
  window.removeEventListener('mouseup', stopDrag);
});

const widgetToggles: Record<string, string> = {
  controlPanel:     'Control Panel',
  lootFeedSession:  'Loot Feed (Session)',
  lootFeedMap:      'Loot Feed (Map)',
  mapTimer:         'Map Timer',
  mapExpense:       'Map Expenses',
  statNetProfitHr:  'Profit / Hr',
  statMapProfit:    'Map Profit',
  statMapsHr:       'Maps / Hr',
  statSessionTime:  'Session Time',
  statInvValue:     'Inventory',
};

function isWidgetVisible(key: string): boolean {
  return (store.widgetPositions as any)[key]?.show ?? false;
}

function toggleWidget(key: string) {
  store.toggleWidget(key);
  store.saveConfigToStorage();
}
</script>

<style scoped>
.builder-action-btn {
  display: flex; align-items: center; gap: 8px;
  width: 100%; padding: 7px 10px; border-radius: 9px;
  font-size: 12px; font-weight: 700; cursor: pointer;
  text-align: left; transition: filter 0.15s;
  color: var(--btn-color); background: var(--btn-bg); border: 1px solid var(--btn-border);
}
.builder-action-btn:hover { filter: brightness(1.3); }

.builder-slide-enter-active, .builder-slide-leave-active {
  transition: opacity 0.18s, transform 0.18s;
  transform-origin: center center;
}
.builder-slide-enter-from, .builder-slide-leave-to {
  opacity: 0; transform: scale(0.94);
}
</style>
