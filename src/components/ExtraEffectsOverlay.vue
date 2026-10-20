<template>
  <div class="fixed inset-0 pointer-events-none z-[60]">

    <Transition name="pulse-enter">
      <div
        v-if="store.dropFxStyle === 'classic' && pulse.active"
        class="absolute pointer-events-none select-none flex flex-col items-center gap-[5px]"
        :style="{
          left:      pulse.x + 'px',
          top:       pulse.y + 'px',
          transform: `translate(-50%, -50%) scale(${pulse.scale * store.fxScale})`,
          opacity:   pulse.opacity,
          transition: 'none',
        }"
      >
        <div
          class="font-black leading-none tracking-tight"
          :class="{ 'rainbow-text': pulse.isRainbow }"
          :style="{
            fontFamily: '\'Segoe UI Black\', Impact, sans-serif',
            fontSize:   '56px',
            color:      pulse.color,
            textShadow: '2px 2px 0 rgba(0,0,0,0.9), -2px -2px 0 rgba(0,0,0,0.9), 2px -2px 0 rgba(0,0,0,0.9), -2px 2px 0 rgba(0,0,0,0.9)',
          }"
        >{{ pulse.total.toFixed(1) }} <span style="font-size:0.55em; opacity:0.75;">FE</span></div>

        <div
          v-for="(h, i) in pulse.history"
          :key="i"
          class="font-bold leading-tight"
          :style="{
            fontFamily: '\'Segoe UI\', sans-serif',
            fontSize:   '14px',
            color:      '#ffffff',
            opacity:    1 - i * 0.12,
            textShadow: '1px 1px 3px rgba(0,0,0,0.9)',
          }"
        >+{{ h.toFixed(1) }}</div>
      </div>
    </Transition>

    <Transition name="pulse-enter">
      <div
        v-if="store.dropFxStyle === 'modern' && pulse.active"
        class="absolute pointer-events-none select-none"
        :style="{
          left:      pulse.x + 'px',
          top:       pulse.y + 'px',
          transform: `translate(-50%, -50%) scale(${pulse.scale * store.fxScale})`,
          opacity:   pulse.opacity,
          transition: 'none',
          minWidth:  '220px',
        }"
      >
        <div
          class="absolute -inset-3 rounded-[28px] blur-xl opacity-40"
          :style="{ background: `radial-gradient(ellipse, ${pulse.color}88 0%, transparent 70%)` }"
        />

        <div
          class="relative flex flex-col items-center gap-2 px-7 py-4 rounded-3xl overflow-hidden"
          :style="{
            background:    'linear-gradient(145deg, rgba(8,8,14,0.90) 0%, rgba(18,16,28,0.95) 100%)',
            backdropFilter: 'blur(16px)',
            border:        `1px solid ${pulse.color}45`,
            boxShadow:     `0 0 0 1px ${pulse.color}18, 0 8px 32px rgba(0,0,0,0.7), 0 0 40px ${pulse.color}22, inset 0 0 20px ${pulse.color}0a`,
          }"
        >
          <div
            class="absolute top-0 left-0 right-0 h-[1.5px] rounded-t-3xl"
            :style="{ background: `linear-gradient(90deg, transparent 0%, ${pulse.color}cc 40%, ${pulse.color} 50%, ${pulse.color}cc 60%, transparent 100%)` }"
          />

          <div
            class="absolute -top-10 left-1/2 -translate-x-1/2 w-32 h-20 rounded-full opacity-20 blur-2xl"
            :style="{ background: pulse.color }"
          />

          <div class="relative flex items-baseline gap-2 z-10">
            <span
              class="font-black leading-none tracking-tighter"
              :class="{ 'rainbow-text': pulse.isRainbow }"
              :style="{
                fontFamily: '\'Segoe UI Black\', Impact, sans-serif',
                fontSize:   '54px',
                color:      pulse.color,
                textShadow: '2px 2px 0 rgba(0,0,0,0.9), -2px -2px 0 rgba(0,0,0,0.9), 2px -2px 0 rgba(0,0,0,0.9), -2px 2px 0 rgba(0,0,0,0.9)',
              }"
            >{{ pulse.total.toFixed(1) }}</span>
            <span
              class="font-bold text-base tracking-wide"
              :style="{ color: pulse.color, opacity: 0.65 }"
            >FE</span>
          </div>

          <div
            class="relative z-10 flex items-center gap-1 px-3 py-[2px] rounded-full text-[10px] font-bold uppercase tracking-[3px]"
            :style="{
              background: `${pulse.color}1a`,
              border:     `1px solid ${pulse.color}40`,
              color:      pulse.color,
              opacity:    0.85,
            }"
          >{{ tierLabel }}</div>

          <div
            class="relative z-10 w-full h-px"
            :style="{ background: `linear-gradient(90deg, transparent, ${pulse.color}55, transparent)` }"
          />

          <div class="relative z-10 flex flex-col items-center gap-[3px]">
            <div
              v-for="(h, i) in pulse.history"
              :key="i"
              class="text-[13px] font-semibold"
              :style="{
                color:   '#e0e0e0',
                opacity: 1 - i * 0.13,
                fontFamily: '\'Segoe UI\', sans-serif',
                textShadow: '0 1px 3px rgba(0,0,0,0.8)',
              }"
            >+{{ h.toFixed(1) }}</div>
          </div>
        </div>
      </div>
    </Transition>

    <DraggableWidget
      v-if="store.extraFxStyle === 'piggy' && store.piggyTier >= 1"
      widgetId="piggyBank"
      title="🐷 Piggy Bank"
      :noBackground="true"
      :defaultPosition="{ x: 700, y: 200 }"
    >
      <div class="w-full h-full">
        <PiggyWidget />
      </div>
    </DraggableWidget>

    <template v-if="store.extraFxStyle === 'casino'">

      <div
        v-for="ci in casinoIcons"
        :key="ci.id"
        class="absolute will-change-transform pointer-events-none"
        :style="{
          left:    ci.x + 'px',
          top:     ci.y + 'px',
          width:   ci.size + 'px',
          height:  ci.size + 'px',
          opacity: ci.opacity * casinoOpacity,
          transition: 'none',
        }"
      >
        <AssetImage
          :src="getIcon(ci.item_id)"
          class="w-full h-full object-contain"
        />
      </div>

      <Transition name="win-msg">
        <div
          v-if="winMsg"
          :key="winMsg.id"
          class="absolute left-1/2 top-1/2"
          :style="{
            transform:     'translate(-50%, -50%)',
            fontSize:      winMsg.fontSize + 'px',
            fontFamily:    '\'Impact\', \'Segoe UI Black\', sans-serif',
            letterSpacing: '4px',
            color:         winMsg.color,
            textShadow:    `0 0 20px ${winMsg.color}, 0 0 40px ${winMsg.color}, 3px 3px 0 #000`,
            userSelect:    'none',
            opacity:       casinoOpacity,
            whiteSpace:    'nowrap',
          }"
          :class="{ 'rainbow-text': winMsg.isRainbow }"
        >{{ winMsg.text }}</div>
      </Transition>

      <div
        v-for="cf in confettiIcons"
        :key="cf.id"
        class="absolute will-change-transform pointer-events-none"
        :style="{
          left:    cf.x + 'px',
          top:     cf.y + 'px',
          width:   cf.size + 'px',
          height:  cf.size + 'px',
          opacity: cf.opacity,
          transition: 'none',
        }"
      >
        <AssetImage
          :src="getIcon(cf.item_id)"
          class="w-full h-full object-contain"
        />
      </div>

    </template>

    <AssetImage
      v-for="p in coinParticles"
      :key="p.id"
      :src="getAnim(`coin/coin_${p.frame}`)"
      draggable="false"
      class="absolute will-change-transform pointer-events-none select-none"
      :style="{
        width:      '416px',
        height:     'auto',
        left:       (p.x - 210) + 'px',
        top:        (p.y - 155) + 'px',
        opacity:    p.opacity,
        transition: 'none',
      }"
    />

    <div
      v-for="p in circleParticles"
      :key="p.id"
      class="absolute rounded-full will-change-transform"
      :style="{
        left:      p.x + 'px',
        top:       p.y + 'px',
        width:     p.size + 'px',
        height:    p.size + 'px',
        background: 'radial-gradient(circle, #fbd786 0%, #f7797d 100%)',
        opacity:   p.opacity,
        boxShadow: '0 0 6px 2px rgba(247,121,125,0.6)',
      }"
    />
  </div>
</template>

<script setup lang="ts">
import AssetImage from './AssetImage.vue';
import { ref, computed, watch, onUnmounted } from 'vue';
import { useTrackerStore, type LootItem } from '../stores/trackerStore';
import { getIcon, getAnim } from '../utils/assets';
import PiggyWidget      from './PiggyWidget.vue';
import DraggableWidget  from './DraggableWidget.vue';

const store = useTrackerStore();

interface PulseDisplay {
  active: boolean; total: number; history: number[];
  x: number; y: number; color: string; isRainbow: boolean;
  opacity: number; scale: number; life: number;
}
interface WinMsg { id: number; text: string; color: string; fontSize: number; isRainbow: boolean; }
interface CoinParticle { id: number; x: number; y: number; speed: number; opacity: number; frame: number; }
interface CircleParticle { id: number; x: number; y: number; size: number; speed: number; opacity: number; }

interface CasinoIcon {
  id: number;
  item_id: string;
  x: number;
  y: number;
  startY: number;
  targetY: number;
  size: number;
  elapsed: number;
  duration: number;
  landed: boolean;
  opacity: number;
}

interface ConfettiIcon {
  id: number;
  item_id: string;
  x: number; y: number;
  size: number;
  speedY: number; speedX: number;
  opacity: number;
}

const pulse = ref<PulseDisplay>({
  active: false, total: 0, history: [],
  x: 0, y: 0, color: '#FFFFFF', isRainbow: false,
  opacity: 1, scale: 1, life: 0,
});
const winMsg          = ref<WinMsg | null>(null);
const coinParticles   = ref<CoinParticle[]>([]);
const circleParticles = ref<CircleParticle[]>([]);

const casinoIcons    = ref<CasinoIcon[]>([]);
const confettiIcons  = ref<ConfettiIcon[]>([]);
const casinoOpacity  = ref(1);

let casinoMatrix: string[][] = [];
let casinoMatchChecked = false;
let casinoFadingOut    = false;
let casinoHideTimeout: ReturnType<typeof setTimeout> | null = null;

const COIN_TOTAL_FRAMES  = 122;
const MAX_COINS          = 25;
const CASINO_COLS        = 6;
const CASINO_MAX_ITEMS   = CASINO_COLS * 8;
const CASINO_ICON_SIZE   = 64;
const CASINO_SPACING     = 5;
const CASINO_THRESHOLD   = 100;
const pulseLifeFrames = computed(() => Math.round(store.fxDuration * 60));

let coinId = 0, casinoId = 0, winMsgId = 0;

function colorForValue(v: number): string {
  if (v < 10)  return '#FFFFFF';
  if (v < 30)  return '#00FF00';
  if (v < 50)  return '#FF00FF';
  if (v < 100) return '#FF0000';
  return '#FF00FF';
}

const tierLabel = computed(() => {
  const t = pulse.value.total;
  if (t >= 100) return '✦ LEGENDARY';
  if (t >= 50)  return '◆ EPIC';
  if (t >= 30)  return '▲ RARE';
  if (t >= 10)  return '● UNCOMMON';
  return '· COMMON';
});

function easeOutBounce(t: number): number {
  const n1 = 7.5625, d1 = 2.75;
  if (t < 1 / d1)        return n1 * t * t;
  if (t < 2 / d1)        { t -= 1.5 / d1;   return n1 * t * t + 0.75; }
  if (t < 2.5 / d1)      { t -= 2.25 / d1;  return n1 * t * t + 0.9375; }
                 { t -= 2.625 / d1; return n1 * t * t + 0.984375; }
}

function triggerPulse(drop: LootItem) {
  const value = drop.price * drop.qty;
  if (value < 1) return;
  const p = pulse.value;
  p.total    += value;
  p.history.push(value);
  if (p.history.length > 7) p.history.shift();
  p.color     = colorForValue(p.total);
  p.isRainbow = p.total >= 100;
  p.life      = pulseLifeFrames.value;
  p.opacity   = 1.0;
  p.scale     = 1.3;
  if (!p.active) {
    p.x = window.innerWidth  * (store.fxX / 100) + (Math.random() * 2 - 1) * 30;
    p.y = window.innerHeight * (store.fxY / 100) + (Math.random() * 2 - 1) * 30;
  }
  p.active = true;
}

function spawnCoins(count: number) {
  const n = Math.min(count, MAX_COINS - coinParticles.value.length);
  for (let i = 0; i < n; i++) {
    coinParticles.value.push({
      id: coinId++,
      x:  Math.random() * window.innerWidth,
      y:  -155 + (Math.random() * 40 - 20),
      speed:   10 + Math.random() * 12,
      opacity: 1,
      frame:   Math.floor(Math.random() * COIN_TOTAL_FRAMES),
    });
  }
}

function spawnCasino(mapDrops: LootItem[]) {
  if (casinoHideTimeout !== null) { clearTimeout(casinoHideTimeout); casinoHideTimeout = null; }

  casinoIcons.value   = [];
  confettiIcons.value = [];
  casinoMatrix        = [];
  casinoOpacity.value = 1;
  casinoMatchChecked  = false;
  casinoFadingOut     = false;
  winMsg.value        = null;

  const source = mapDrops.slice(0, CASINO_MAX_ITEMS).reverse();
  if (!source.length) return;

  const cols       = CASINO_COLS;
  const actualRows = Math.ceil(source.length / cols);
  const gridW      = cols       * (CASINO_ICON_SIZE + CASINO_SPACING);
  const gridH      = actualRows * (CASINO_ICON_SIZE + CASINO_SPACING);
  const gridStartX = Math.round(window.innerWidth  / 2 - gridW / 2);
  const gridStartY = Math.round(window.innerHeight / 2 - gridH / 2);

  casinoMatrix = Array.from({ length: actualRows }, () => Array(cols).fill(''));

  source.forEach((drop, i) => {
    const row = Math.floor(i / cols);
    const col = i % cols;
    casinoMatrix[row][col] = drop.item_id;

    const targetX  = gridStartX + col * (CASINO_ICON_SIZE + CASINO_SPACING);
    const targetY  = gridStartY + row * (CASINO_ICON_SIZE + CASINO_SPACING);
    const startY   = -(window.innerHeight + Math.random() * 500);
    const duration = 0.8 + row * 0.1;

    casinoIcons.value.push({
      id: casinoId++,
      item_id: drop.item_id,
      x: targetX,
      y: startY,
      startY,
      targetY,
      size: CASINO_ICON_SIZE,
      elapsed: 0,
      duration,
      landed: false,
      opacity: 1,
    });
  });
}

function checkCasinoMatches() {
  casinoMatchChecked = true;

  const grid  = casinoMatrix;
  const rows  = grid.length;
  const cols  = grid[0]?.length ?? 0;
  const matches = new Set<string>();

  function getId(r: number, c: number): string | null {
    if (r < 0 || r >= rows || c < 0 || c >= cols) return null;
    return grid[r][c] || null;
  }

  for (let r = 0; r < rows; r++)
    for (let c = 0; c <= cols - 3; c++) {
      const [a, b, cc] = [getId(r,c), getId(r,c+1), getId(r,c+2)];
      if (a && a === b && b === cc) {
        matches.add(`${r},${c}`); matches.add(`${r},${c+1}`); matches.add(`${r},${c+2}`);
      }
    }

  for (let c = 0; c < cols; c++)
    for (let r = 0; r <= rows - 3; r++) {
      const [a, b, cc] = [getId(r,c), getId(r+1,c), getId(r+2,c)];
      if (a && a === b && b === cc) {
        matches.add(`${r},${c}`); matches.add(`${r+1},${c}`); matches.add(`${r+2},${c}`);
      }
    }

  for (let r = 0; r <= rows - 3; r++)
    for (let c = 0; c <= cols - 3; c++) {
      const [a, b, cc] = [getId(r,c), getId(r+1,c+1), getId(r+2,c+2)];
      if (a && a === b && b === cc) {
        matches.add(`${r},${c}`); matches.add(`${r+1},${c+1}`); matches.add(`${r+2},${c+2}`);
      }
    }

  for (let r = 2; r < rows; r++)
    for (let c = 0; c <= cols - 3; c++) {
      const [a, b, cc] = [getId(r,c), getId(r-1,c+1), getId(r-2,c+2)];
      if (a && a === b && b === cc) {
        matches.add(`${r},${c}`); matches.add(`${r-1},${c+1}`); matches.add(`${r-2},${c+2}`);
      }
    }

  if (matches.size > 0) {
    const mc = matches.size;
    let text: string, color: string, fontSize: number, isRainbow: boolean;
    if      (mc < 5)  { text = 'NICE WIN!';  color = '#00FF00'; fontSize = 44; isRainbow = false; }
    else if (mc < 10) { text = 'BIG WIN!';   color = '#FF6600'; fontSize = 52; isRainbow = false; }
    else if (mc < 15) { text = 'MEGA WIN!';  color = '#FF00FF'; fontSize = 64; isRainbow = false; }
    else              { text = 'JACKPOT!!!'; color = '#FFD700'; fontSize = 80; isRainbow = true;  }
    winMsg.value = { id: winMsgId++, text, color, fontSize, isRainbow };
    spawnConfetti();
  }

  casinoHideTimeout = setTimeout(() => { casinoFadingOut = true; }, 3000);
}

function spawnConfetti() {
  const drops = store.currentMapDrops;
  if (!drops.length) return;
  for (let i = 0; i < 30; i++) {
    const drop = drops[Math.floor(Math.random() * drops.length)];
    confettiIcons.value.push({
      id: casinoId++,
      item_id: drop.item_id,
      x:      Math.random() * window.innerWidth,
      y:      -80,
      size:   30 + Math.random() * 50,
      speedY: 3 + Math.random() * 6,
      speedX: (Math.random() - 0.5) * 3,
      opacity: 1,
    });
  }
}

watch(() => store.latestDrop, (drop: LootItem | null) => {
  if (!drop) return;
  const value = drop.price * drop.qty;
  if (!store.shouldAnimateDrop(drop)) return;

  if (store.dropFxStyle !== 'none') triggerPulse(drop);

  if (store.extraFxStyle === 'casino' && value >= CASINO_THRESHOLD)
    spawnCasino(store.currentMapDrops);

  if (store.extraFxStyle === 'piggy' && store.piggyTier >= 3)
    spawnCoins(3);

  startLoop();
});

let rafId = 0, frameCount = 0;

function hasActiveParticles(): boolean {
  return pulse.value.active ||
    coinParticles.value.length > 0 ||
    circleParticles.value.length > 0 ||
    casinoIcons.value.length > 0 ||
    confettiIcons.value.length > 0;
}

function startLoop() {
  if (rafId === 0) rafId = requestAnimationFrame(loop);
}

function loop() {
  frameCount++;

  for (let i = circleParticles.value.length - 1; i >= 0; i--) {
    const p = circleParticles.value[i];
    p.y += p.speed; p.opacity -= 0.008;
    if (p.y > window.innerHeight + 80 || p.opacity <= 0) circleParticles.value.splice(i, 1);
  }

  for (let i = coinParticles.value.length - 1; i >= 0; i--) {
    const p = coinParticles.value[i];
    p.y += p.speed;
    if (frameCount % 2 === 0) p.frame = (p.frame + 2) % COIN_TOTAL_FRAMES;
    if (p.y > window.innerHeight + 155) coinParticles.value.splice(i, 1);
  }

  if (pulse.value.active) {
    const p = pulse.value;
    p.life--;
    if (p.scale > 1.0) p.scale = Math.max(1.0, p.scale - 0.022);
    if (p.life < 60)   p.opacity = p.life / 60;
    if (p.life <= 0) {
      p.active = false; p.total = 0; p.history = [];
      p.opacity = 1; p.scale = 1;
    }
  }

  let allLanded = casinoIcons.value.length > 0;
  for (const ci of casinoIcons.value) {
    if (!ci.landed) {
      ci.elapsed += 1 / 60;
      const t = Math.min(ci.elapsed / ci.duration, 1.0);
      ci.y = ci.startY + (ci.targetY - ci.startY) * easeOutBounce(t);
      if (t >= 1.0) { ci.y = ci.targetY; ci.landed = true; }
      else          { allLanded = false; }
    }
  }

  if (allLanded && !casinoMatchChecked) {
    checkCasinoMatches();
  }

  if (casinoFadingOut) {
    casinoOpacity.value = Math.max(0, casinoOpacity.value - 0.02);
    if (casinoOpacity.value <= 0) {
      casinoFadingOut    = false;
      casinoIcons.value  = [];
      winMsg.value       = null;
      casinoOpacity.value = 1;
      casinoMatchChecked  = false;
    }
  }

  for (let i = confettiIcons.value.length - 1; i >= 0; i--) {
    const cf = confettiIcons.value[i];
    cf.y += cf.speedY;
    cf.x += cf.speedX;
    if (cf.y > window.innerHeight + 80) confettiIcons.value.splice(i, 1);
  }

  if (hasActiveParticles()) {
    rafId = requestAnimationFrame(loop);
  } else {
    rafId = 0;
  }
}
onUnmounted(() => {
  if (rafId) cancelAnimationFrame(rafId);
  if (casinoHideTimeout !== null) clearTimeout(casinoHideTimeout);
});
</script>

<style scoped>
.pulse-enter-enter-active { transition: all 0.15s cubic-bezier(0.34, 1.56, 0.64, 1); }
.pulse-enter-leave-active { transition: all 0.3s ease-out; }
.pulse-enter-enter-from   { opacity: 0; transform: translate(-50%, -50%) scale(0.6); }
.pulse-enter-leave-to     { opacity: 0; transform: translate(-50%, -50%) scale(1.1); }

.win-msg-enter-active { transition: all 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
.win-msg-leave-active { transition: all 0.5s ease-out; }
.win-msg-enter-from   { opacity: 0; transform: translate(-50%, -50%) scale(0.3); }
.win-msg-leave-to     { opacity: 0; transform: translate(-50%, -50%) scale(1.4); }

@keyframes rainbow {
  0%   { color: #ff0000; }
  15%  { color: #ff7f00; }
  30%  { color: #ffff00; }
  45%  { color: #00ff00; }
  60%  { color: #0000ff; }
  75%  { color: #8b00ff; }
  100% { color: #ff0000; }
}
.rainbow-text {
  animation: rainbow 2s linear infinite;
  text-shadow:
    2px  2px 0 rgba(0,0,0,0.8),
   -2px -2px 0 rgba(0,0,0,0.8),
    2px -2px 0 rgba(0,0,0,0.8),
   -2px  2px 0 rgba(0,0,0,0.8) !important;
}
</style>
