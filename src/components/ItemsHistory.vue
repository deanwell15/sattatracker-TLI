<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-[150] flex items-center justify-center pointer-events-auto"
    style="background: rgba(0,0,0,0.8);"
    @click.self="$emit('close')"
  >
    <div
      class="flex overflow-hidden select-none"
      style="
        width: 860px; height: 600px;
        background: linear-gradient(160deg, rgba(14,12,22,0.98) 0%, rgba(10,9,18,0.99) 100%);
        border: 1px solid rgba(255,255,255,0.09);
        border-radius: 18px;
        box-shadow: 0 24px 80px rgba(0,0,0,0.85), inset 0 1px 0 rgba(255,255,255,0.06);
      "
    >
      <div
        class="flex flex-col shrink-0 py-3"
        style="width: 175px; border-right: 1px solid rgba(255,255,255,0.06);"
      >
        <div class="px-4 pb-3" style="border-bottom: 1px solid rgba(255,255,255,0.05);">
          <div class="text-[11px] font-black uppercase tracking-[3px]" style="color: rgba(255,255,255,0.55);">Inventory</div>
          <div class="text-[10px] mt-[2px] font-mono" style="color: rgba(255,255,255,0.2);">
            {{ store.inventoryValue }} FE total
          </div>
        </div>

        <div class="flex flex-col gap-[2px] px-2 pt-3">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="flex items-center gap-[8px] px-3 py-[8px] rounded-[9px] text-left transition-all duration-150"
            :style="{
              background: activeTab === tab.id ? 'rgba(255,255,255,0.07)' : 'transparent',
              borderLeft: activeTab === tab.id ? '2px solid rgba(96,165,250,0.7)' : '2px solid transparent',
            }"
            @click="activeTab = tab.id"
          >
            <span class="text-[15px]">{{ tab.icon }}</span>
            <div>
              <div class="text-[12px] font-semibold leading-tight" :style="{ color: activeTab === tab.id ? 'rgba(255,255,255,0.9)' : 'rgba(255,255,255,0.45)' }">
                {{ tab.label }}
              </div>
              <div class="text-[10px]" style="color: rgba(255,255,255,0.2);">{{ tabCount(tab.id) }} items</div>
            </div>
          </button>
        </div>

        <div class="flex-1" />

        <div class="px-2 pb-1">
          <button
            @click="$emit('close')"
            class="w-full py-2 rounded-[9px] text-[12px] font-bold transition-all duration-150"
            style="background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.25); color: #f87171;"
            onmouseover="this.style.background='rgba(248,113,113,0.18)'"
            onmouseout="this.style.background='rgba(248,113,113,0.1)'"
          >Close</button>
        </div>
      </div>

      <div class="flex-1 flex flex-col min-w-0">

        <div class="flex items-center gap-3 px-5 py-3" style="border-bottom: 1px solid rgba(255,255,255,0.06);">
          <div class="text-[15px] font-bold" style="color: rgba(255,255,255,0.8);">{{ currentTabLabel }}</div>

          <div class="flex-1" />

          <input
            v-if="activeTab === 'prices'"
            v-model="priceSearch"
            type="text"
            placeholder="Search by name or ID..."
            class="text-[12px] px-3 py-[5px] rounded-lg outline-none"
            style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); color: rgba(255,255,255,0.8); width: 200px;"
          />

          <div v-if="activeTab !== 'prices'" class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-[2px]" style="color: rgba(255,255,255,0.25);">Sort</span>
            <button
              class="text-[11px] font-semibold px-3 py-1 rounded-lg transition-all duration-150"
              :style="sortBy === 'price' ? activeToggleStyle : inactiveToggleStyle"
              @click="sortBy = 'price'"
            >By Value</button>
            <button
              class="text-[11px] font-semibold px-3 py-1 rounded-lg transition-all duration-150"
              :style="sortBy === 'count' ? activeToggleStyle : inactiveToggleStyle"
              @click="sortBy = 'count'"
            >By Count</button>
          </div>

          <div v-if="activeTab !== 'prices'" class="flex items-center gap-1">
            <button
              class="w-7 h-7 flex items-center justify-center rounded-lg transition-all duration-150 text-[14px]"
              :style="viewMode === 'list' ? activeToggleStyle : inactiveToggleStyle"
              @click="viewMode = 'list'"
              title="List view"
            >☰</button>
            <button
              class="w-7 h-7 flex items-center justify-center rounded-lg transition-all duration-150 text-[14px]"
              :style="viewMode === 'grid' ? activeToggleStyle : inactiveToggleStyle"
              @click="viewMode = 'grid'"
              title="Grid view"
            >⊞</button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto custom-scroll px-4 py-3" @contextmenu.prevent>

          <template v-if="activeTab === 'prices'">
            <div v-if="filteredPrices.length === 0" class="flex items-center justify-center h-full">
              <div class="text-center">
                <div class="text-[32px] mb-2 opacity-30">💰</div>
                <div class="text-[13px]" style="color: rgba(255,255,255,0.25);">No price data yet</div>
              </div>
            </div>
            <div v-else class="flex flex-col gap-[4px]">
              <div
                v-for="item in filteredPrices"
                :key="item.item_id"
                class="flex items-center gap-3 px-3 py-[7px] rounded-xl"
                style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);"
              >
                <div
                  class="w-9 h-9 rounded-lg shrink-0 flex items-center justify-center overflow-hidden"
                  style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);"
                >
                  <AssetImage
                    :src="getIcon(item.item_id)"
                    class="w-7 h-7 object-contain"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-[13px] font-semibold truncate" style="color: rgba(255,255,255,0.85);">{{ item.item_name }}</div>
                  <div class="text-[10px]" style="color: rgba(255,255,255,0.2);">ID: {{ item.item_id }}</div>
                </div>
                <div class="text-[13px] font-bold shrink-0" style="color: #fbbf24;">{{ item.price.toFixed(2) }} FE</div>
              </div>
            </div>
          </template>

          <div v-else-if="displayData.length === 0" class="flex items-center justify-center h-full">
            <div class="text-center px-4">
              <div class="text-[32px] mb-2 opacity-30">{{ activeTab === 'inventory' ? '🎒' : '📦' }}</div>
              <div class="text-[13px]" style="color: rgba(255,255,255,0.25);">
                {{ activeTab === 'inventory' ? 'No inventory data yet' : 'No items' }}
              </div>
              <div v-if="activeTab === 'inventory'" class="text-[11px] mt-2" style="color: rgba(255,255,255,0.15);">
                Sort your inventory in-game to populate this tab
              </div>
            </div>
          </div>

          <template v-else-if="viewMode === 'list'">
            <div class="flex flex-col gap-[4px]">
              <div
                v-for="item in displayData"
                :key="item.item_id"
                class="flex items-center gap-3 px-3 py-[7px] rounded-xl cursor-pointer transition-colors duration-150"
                style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);"
                @mouseenter="($event.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.06)'"
                @mouseleave="($event.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.03)'"
                @contextmenu.prevent="openContextMenu($event, item)"
              >
                <div
                  class="w-9 h-9 rounded-lg shrink-0 flex items-center justify-center overflow-hidden"
                  style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);"
                >
                  <AssetImage
                    :src="getIcon(item.item_id)"
                    class="w-7 h-7 object-contain"
                  />
                </div>

                <div class="flex-1 min-w-0">
                  <div
                    class="text-[13px] font-semibold truncate"
                    :style="{ color: store.isBlocked(item.item_id) ? 'rgba(255,255,255,0.3)' : 'rgba(255,255,255,0.85)' }"
                  >{{ item.item_name }}</div>
                  <div class="text-[10px]" style="color: rgba(255,255,255,0.2);">ID: {{ item.item_id }}</div>
                </div>

                <div v-if="store.isBlocked(item.item_id)" class="text-[9px] px-2 py-[2px] rounded-full font-bold" style="background: rgba(248,113,113,0.15); color: #f87171; border: 1px solid rgba(248,113,113,0.25);">BLOCKED</div>

                <div class="text-right shrink-0">
                  <div class="text-[13px] font-bold" style="color: #fbbf24;">
                    {{ store.isBlocked(item.item_id) ? '—' : (item.price * item.qty).toFixed(1) + ' FE' }}
                  </div>
                  <div class="text-[10px]" style="color: rgba(255,255,255,0.3);">x{{ item.qty }} @ {{ item.price.toFixed(2) }}</div>
                </div>
              </div>
            </div>
          </template>

          <template v-else>
            <div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));">
              <div
                v-for="item in displayData"
                :key="item.item_id"
                class="flex flex-col items-center gap-[6px] p-3 rounded-xl cursor-pointer transition-colors duration-150 text-center"
                style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.05);"
                @mouseenter="($event.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.06)'"
                @mouseleave="($event.currentTarget as HTMLElement).style.background = 'rgba(255,255,255,0.03)'"
                @contextmenu.prevent="openContextMenu($event, item)"
              >
                <div
                  class="w-10 h-10 rounded-lg flex items-center justify-center overflow-hidden"
                  style="background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.08);"
                >
                  <AssetImage
                    :src="getIcon(item.item_id)"
                    class="w-8 h-8 object-contain"
                  />
                </div>
                <div class="text-[10px] font-semibold leading-tight w-full truncate" style="color: rgba(255,255,255,0.7);">{{ item.item_name }}</div>
                <div class="text-[11px] font-bold" style="color: #fbbf24;">
                  {{ store.isBlocked(item.item_id) ? '—' : (item.price * item.qty).toFixed(0) + ' FE' }}
                </div>
                <div class="text-[10px]" style="color: rgba(255,255,255,0.3);">x{{ item.qty }}</div>
              </div>
            </div>
          </template>

        </div>
      </div>
    </div>

    <div
      v-if="ctxMenu.visible"
      class="fixed z-[200] py-1 rounded-xl overflow-hidden"
      style="
        min-width: 180px;
        background: linear-gradient(160deg, rgba(20,18,32,1) 0%, rgba(14,12,22,1) 100%);
        border: 1px solid rgba(255,255,255,0.1);
        box-shadow: 0 12px 40px rgba(0,0,0,0.8);
      "
      :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
      @click.stop
    >
      <div class="px-3 py-[6px]" style="border-bottom: 1px solid rgba(255,255,255,0.06);">
        <div class="text-[11px] font-semibold truncate" style="color: rgba(255,255,255,0.5); max-width: 160px;">{{ ctxMenu.item?.item_name }}</div>
      </div>
      <template v-if="ctxMenu.item">
        <button
          v-if="!store.isBlocked(ctxMenu.item.item_id)"
          class="w-full text-left px-3 py-[8px] text-[12px] font-semibold transition-colors duration-100"
          style="color: #f87171; background: transparent;"
          onmouseover="this.style.background='rgba(248,113,113,0.1)'"
          onmouseout="this.style.background='transparent'"
          @click="blockItem(ctxMenu.item!.item_id)"
        >🚫 Block item (price → 0)</button>
        <button
          v-else
          class="w-full text-left px-3 py-[8px] text-[12px] font-semibold transition-colors duration-100"
          style="color: #4ade80; background: transparent;"
          onmouseover="this.style.background='rgba(74,222,128,0.1)'"
          onmouseout="this.style.background='transparent'"
          @click="unblockItem(ctxMenu.item!.item_id)"
        >✓ Unblock item (restore price)</button>
      </template>
    </div>

    <div v-if="ctxMenu.visible" class="fixed inset-0 z-[199]" @click="ctxMenu.visible = false" @contextmenu.prevent="ctxMenu.visible = false" />
  </div>
</template>

<script setup lang="ts">
import AssetImage from './AssetImage.vue';
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useTrackerStore, type LootItem } from '../stores/trackerStore';
import { getIcon } from '../utils/assets';

defineProps<{ isOpen: boolean }>();
defineEmits(['close']);

const store = useTrackerStore();

const activeTab = ref<'session' | 'map' | 'inventory' | 'blocked' | 'prices'>('session');
const sortBy    = ref<'price' | 'count'>('price');
const viewMode  = ref<'list' | 'grid'>('list');
const priceSearch = ref('');

interface PriceEntry { item_id: string; item_name: string; price: number; }
const allPrices = ref<PriceEntry[]>([]);

async function loadPrices() {
  try {
    allPrices.value = await invoke<PriceEntry[]>('get_all_prices');
  } catch (e) {
    console.error('Failed to load prices', e);
  }
}

watch(activeTab, (tab) => {
  if (tab === 'prices') loadPrices();
});

const filteredPrices = computed(() => {
  const q = priceSearch.value.trim().toLowerCase();
  if (!q) return allPrices.value;
  return allPrices.value.filter(p =>
    p.item_name.toLowerCase().includes(q) || p.item_id.includes(q)
  );
});

const tabs = [
  { id: 'session',   label: 'Session',   icon: '📋' },
  { id: 'map',       label: 'Last Map',  icon: '🗺️' },
  { id: 'inventory', label: 'Inventory', icon: '🎒' },
  { id: 'blocked',   label: 'Blocked',   icon: '🚫' },
  { id: 'prices',    label: 'All Prices', icon: '💰' },
] as const;

const activeToggleStyle   = 'background: rgba(96,165,250,0.2); border: 1px solid rgba(96,165,250,0.4); color: #60a5fa;';
const inactiveToggleStyle = 'background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.4);';

const currentTabLabel = computed(() => tabs.find(t => t.id === activeTab.value)?.label ?? '');

function aggregateDrops(drops: LootItem[]) {
  const map = new Map<string, LootItem>();
  for (const d of drops) {
    if (!map.has(d.item_id)) map.set(d.item_id, { ...d, qty: 0 });
    map.get(d.item_id)!.qty += d.qty;
  }
  return Array.from(map.values());
}

const blockedData = computed((): LootItem[] =>
  store.blockedItemIds.map(id => {
    const match =
      store.inventoryItems.find(i => i.item_id === id) ||
      store.sessionDrops.find(d => d.item_id === id);
    return {
      item_id:   id,
      item_name: match?.item_name ?? `Item ${id}`,
      qty:       match?.qty ?? 0,
      price:     match?.price ?? 0,
      timestamp: 0,
    };
  })
);

const inventoryData = computed((): LootItem[] =>
  store.inventoryItems.map(i => ({
    item_id:   i.item_id,
    item_name: i.item_name,
    qty:       i.qty,
    price:     i.price,
    timestamp: 0,
  }))
);

const rawData = computed((): LootItem[] => {
  if (activeTab.value === 'session')   return aggregateDrops(store.sessionDrops);
  if (activeTab.value === 'map')       return aggregateDrops(store.currentMapDrops);
  if (activeTab.value === 'inventory') return inventoryData.value;
  if (activeTab.value === 'prices')    return [];
  return blockedData.value;
});

const displayData = computed(() => {
  const data = [...rawData.value];
  if (sortBy.value === 'price') {
    data.sort((a, b) => (b.price * b.qty) - (a.price * a.qty));
  } else {
    data.sort((a, b) => b.qty - a.qty);
  }
  return data;
});

function tabCount(id: string) {
  if (id === 'session')   return aggregateDrops(store.sessionDrops).length;
  if (id === 'map')       return aggregateDrops(store.currentMapDrops).length;
  if (id === 'inventory') return store.inventoryItems.length;
  if (id === 'prices')    return allPrices.value.length;
  return store.blockedItemIds.length;
}

const ctxMenu = ref<{ visible: boolean; x: number; y: number; item: LootItem | null }>({
  visible: false, x: 0, y: 0, item: null,
});

function openContextMenu(e: MouseEvent, item: LootItem) {
  e.preventDefault();
  ctxMenu.value = { visible: true, x: e.clientX, y: e.clientY, item };
}

function blockItem(id: string) {
  store.blockItem(id);
  ctxMenu.value.visible = false;
}

function unblockItem(id: string) {
  store.unblockItem(id);
  ctxMenu.value.visible = false;
}

watch(activeTab, () => { ctxMenu.value.visible = false; });
</script>
