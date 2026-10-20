<template>
  <div
    class="w-full h-full flex items-center justify-center"
    style="background: transparent; min-width: 80px; min-height: 80px;"
    @click="triggerCoin"
  >
    <AssetImage
      :src="frameSrc"
      class="w-full h-full object-contain pointer-events-none select-none"
      draggable="false"
      :style="{
        opacity:   pigOpacity,
        transform: `scale(${bounceScale})`,
        transition: 'none',
      }"
      alt="Piggy Bank"
    />
  </div>
</template>

<script setup lang="ts">
import AssetImage from './AssetImage.vue';
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useTrackerStore } from '../stores/trackerStore';
  import { getAnim } from '../utils/assets';
  const store = useTrackerStore();
const TIER_INFO = [
  { start: 0,  len: 30, baseSize: 100 },
  { start: 30, len: 30, baseSize: 140 },
  { start: 60, len: 30, baseSize: 185 },
  { start: 90, len: 30, baseSize: 250 },
] as const;

const COIN_FRAMES = 122;
const FPS         = 30;
const FRAME_MS    = 1000 / FPS;

const COMBO_TIMEOUT_MS  = 5000;
const IDLE_FADE_DELAY   = 2000;
const BOUNCE_INIT       = 0.025;
const BOUNCE_DECAY      = 0.005;

const comboValue    = ref(0);
const visualTier    = ref(0);
const targetTier    = ref(0);
const currentFrame  = ref(0);
const isCoinAnim    = ref(false);
const pigOpacity    = ref(1.0);
const scaleBoost    = ref(0.0);

let lastPickupTime  = 0;
let lastFrameTime   = 0;
let rafId           = 0;

const tierData    = computed(() => TIER_INFO[visualTier.value]);
const bounceScale = computed(() => 1 + scaleBoost.value);

const frameSrc = computed(() => {
  if (isCoinAnim.value) {
    return getAnim(`coin/coin_${currentFrame.value}`);
  }
  return getAnim(`pig_${tierData.value.start + (currentFrame.value % tierData.value.len)}`);
});

function loop(now: number) {
  rafId = requestAnimationFrame(loop);

  const elapsed = now - lastFrameTime;
  if (elapsed < FRAME_MS) return;
  lastFrameTime = now - (elapsed % FRAME_MS);

  const timeSincePickup = now - lastPickupTime;

  if (lastPickupTime > 0 && timeSincePickup > COMBO_TIMEOUT_MS && comboValue.value > 0) {
    comboValue.value = 0;
    targetTier.value = 0;
    store.piggyTier  = 0;
  }

  if (visualTier.value < targetTier.value) {
    visualTier.value++;
    currentFrame.value = 0;
  } else if (visualTier.value > targetTier.value) {
    visualTier.value--;
    currentFrame.value = 0;
  }

  if (scaleBoost.value > 0) {
    scaleBoost.value = Math.max(0, scaleBoost.value - BOUNCE_DECAY);
  }

  if (lastPickupTime === 0 || timeSincePickup < IDLE_FADE_DELAY) {
    pigOpacity.value = 1.0;
  } else {
    const fade = (timeSincePickup - IDLE_FADE_DELAY) / 2000;
    pigOpacity.value = Math.max(0, 1.0 - fade);
  }

  if (isCoinAnim.value) {
    if (currentFrame.value < COIN_FRAMES - 1) {
      currentFrame.value++;
    } else {
      isCoinAnim.value   = false;
      currentFrame.value = 0;
    }
  } else {
    currentFrame.value = (currentFrame.value + 1) % tierData.value.len;
  }
}

function onDrop(value: number) {
  comboValue.value  += value;
  lastPickupTime     = performance.now();
  scaleBoost.value   = BOUNCE_INIT;
  pigOpacity.value   = 1.0;

  if      (comboValue.value < 50)  targetTier.value = 0;
  else if (comboValue.value < 150) targetTier.value = 1;
  else if (comboValue.value < 300) targetTier.value = 2;
  else                             targetTier.value = 3;

  store.piggyTier = targetTier.value;
}

function triggerCoin() {
  if (!isCoinAnim.value) {
    isCoinAnim.value   = true;
    currentFrame.value = 0;
  }
}

watch(
  () => store.latestDrop,
  (drop) => {
    if (drop && drop.price * drop.qty > 0) {
      onDrop(drop.price * drop.qty);
    }
  }
);

onMounted(() => {
  lastPickupTime = 0;
  rafId = requestAnimationFrame(loop);
});

onUnmounted(() => cancelAnimationFrame(rafId));
</script>
