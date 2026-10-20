<template>
  <DraggableWidget
    :widgetId="widgetId"
    :title="mode === 'map' ? 'Loot (Map)' : 'Loot (Session)'"
    :defaultPosition="defaultPosition"
    :defaultW="320"
    :defaultH="450"
  >
    <div
      class="overflow-hidden flex flex-col custom-scroll"
      style="width: 100%; height: 100%; min-width: 160px; min-height: 80px; font-family: var(--theme-font-main);"
    >
      <div
        class="text-center font-bold text-sm py-[5px] border-b shrink-0 select-none"
        style="
          color: #FFD700;
          font-size: 14px;
          border-color: var(--theme-border-color);
          background-color: var(--theme-bg-widget);
        "
      >
        {{ mode === 'map' ? 'Map' : 'Session' }}: {{ totalValue.toFixed(1) }} FE
      </div>

      <div class="flex-1 overflow-y-auto px-[5px] py-[5px] custom-scroll">
        <div v-if="groupedDrops.length === 0" class="text-center py-6 italic text-sm" style="color: var(--theme-text-qty);">
          Waiting for drops...
        </div>

        <TransitionGroup name="loot" tag="div" class="flex flex-col gap-[3px]">
          <div
            v-for="item in groupedDrops"
            :key="`${item.season_id}:${item.item_id}`"
            class="flex items-center justify-between rounded px-[5px] py-[2px]"
            style="
              background-color: rgba(60, 60, 60, 0.6);
              border-radius: 4px;
              min-height: 28px;
            "
          >
            <div class="shrink-0 w-6 h-6 flex items-center justify-center mr-[5px]" style="background-color: rgba(0,0,0,0.3); border-radius: 3px;">
              <AssetImage
                :src="getIcon(item.item_id)"
                class="w-full h-full object-contain"
              />
            </div>

            <div class="flex-1 truncate font-bold text-[12px] mr-2" style="color: var(--theme-text-main);">
              {{ item.item_name }}
            </div>

            <div :title="priceDetails(item)" class="shrink-0 font-bold text-[13px] whitespace-nowrap" style="color: #00FF00;">
              x{{ item.qty }}&nbsp;&nbsp;[{{ priceLabel(item, item.qty) }}]
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import AssetImage from './AssetImage.vue';
import { computed } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
import { priceLabel, priceDetails } from '../utils/prices';
import { getIcon } from '../utils/assets';
import DraggableWidget from './DraggableWidget.vue';

const store = useTrackerStore();

const props = defineProps({
  mode:            { type: String,  default: 'session' },
  widgetId:        { type: String,  default: 'lootFeedSession' },
  defaultPosition: { type: Object,  default: () => ({ x: 50, y: 200 }) },
});

const totalValue = computed(() =>
  props.mode === 'map' ? store.currentMapProfit : store.totalProfit
);

const groupedDrops = computed(() => {
  const rawDrops = props.mode === 'map' ? store.currentMapDrops : store.sessionDrops;

  return rawDrops.filter(item => store.passesLootFilter(item))
    .slice().sort((a, b) => b.price * b.qty - a.price * a.qty)
    .slice(0, 50);
});
</script>

<style scoped>
.loot-enter-active,
.loot-leave-active {
  transition: all 0.25s ease;
}
.loot-enter-from {
  opacity: 0;
  transform: translateX(-12px);
}
.loot-leave-to {
  opacity: 0;
  transform: translateX(12px);
}

.custom-scroll::-webkit-scrollbar       { width: 4px; }
.custom-scroll::-webkit-scrollbar-thumb { background: var(--theme-border-color); border-radius: 4px; }
</style>
