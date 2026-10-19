<template>
  <Transition name="welcome-fade">
    <div
      v-if="visible"
      data-interactive
      class="fixed inset-0 z-[600] flex items-center justify-center pointer-events-auto"
      style="background: rgba(0,0,0,0.82);"
    >
      <div
        class="relative flex flex-col gap-5 rounded-2xl text-white"
        style="
          width: 480px;
          background: linear-gradient(160deg, rgba(14,12,22,0.99) 0%, rgba(10,9,18,1) 100%);
          border: 1px solid rgba(255,255,255,0.09);
          box-shadow: 0 24px 64px rgba(0,0,0,0.9), inset 0 1px 0 rgba(255,255,255,0.07);
          padding: 28px 32px 24px;
        "
      >
        <div class="flex flex-col items-center gap-2">
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full" style="background:#60a5fa; box-shadow:0 0 10px #60a5fa;"></div>
            <span class="text-[11px] font-bold uppercase tracking-[4px]" style="color:rgba(255,255,255,0.35);">Deanfinite</span>
            <div class="w-2 h-2 rounded-full" style="background:#60a5fa; box-shadow:0 0 10px #60a5fa;"></div>
          </div>
          <h2 class="text-xl font-black tracking-tight" style="color:rgba(255,255,255,0.92);">Добро пожаловать в Deanfinite!</h2>
        </div>

        <div class="h-px w-full" style="background: linear-gradient(90deg, transparent, rgba(96,165,250,0.3), transparent);"></div>

        <div class="flex flex-col gap-3">
          <div class="text-[11px] font-bold uppercase tracking-[3px] mb-1" style="color:rgba(255,255,255,0.3);">Горячие клавиши</div>

          <div class="flex items-center gap-3">
            <kbd class="hotkey">F8</kbd>
            <span class="text-sm" style="color:rgba(255,255,255,0.7);">Открыть меню виджетов и настройки</span>
          </div>
          <div class="flex items-center gap-3">
            <kbd class="hotkey">F7</kbd>
            <span class="text-sm" style="color:rgba(255,255,255,0.7);">Скрыть / показать оверлей</span>
          </div>
        </div>

        <div class="h-px w-full" style="background: linear-gradient(90deg, transparent, rgba(96,165,250,0.3), transparent);"></div>

        <div class="flex flex-col gap-3 rounded-xl px-4 py-4" style="background: rgba(251,191,36,0.06); border: 1px solid rgba(251,191,36,0.2);">
          <div class="flex items-center gap-2">
            <span style="color:#fbbf24; font-size:16px;">⚠</span>
            <span class="text-[12px] font-black uppercase tracking-[2px]" style="color:#fbbf24;">Обязательно синхронизируйте инвентарь</span>
          </div>
          <ol class="flex flex-col gap-2 text-sm" style="color:rgba(255,255,255,0.75); padding-left: 4px; list-style: none;">
            <li class="flex items-start gap-3">
              <span class="flex-shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-black" style="background:rgba(96,165,250,0.2); border:1px solid rgba(96,165,250,0.4); color:#60a5fa;">1</span>
              <span>Нажмите <strong style="color:#fff;">ESC</strong> в игре → <strong style="color:#fff;">Enable log</strong></span>
            </li>
            <li class="flex items-start gap-3">
              <span class="flex-shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-black" style="background:rgba(96,165,250,0.2); border:1px solid rgba(96,165,250,0.4); color:#60a5fa;">2</span>
              <span>Откройте <strong style="color:#fff;">инвентарь</strong></span>
            </li>
            <li class="flex items-start gap-3">
              <span class="flex-shrink-0 w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-black" style="background:rgba(96,165,250,0.2); border:1px solid rgba(96,165,250,0.4); color:#60a5fa;">3</span>
              <span>Нажмите кнопку <strong style="color:#fff;">сортировки</strong> — трекер синхронизируется</span>
            </li>
          </ol>
        </div>

        <button
          @click="close"
          :disabled="countdown > 0"
          class="w-full py-[10px] rounded-xl font-bold text-sm transition-all"
          :style="countdown > 0
            ? 'background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.25); cursor: not-allowed;'
            : 'background: rgba(96,165,250,0.18); border: 1px solid rgba(96,165,250,0.4); color: #60a5fa; cursor: pointer;'"
        >
          {{ countdown > 0 ? `Закрыть (${countdown})` : 'Понял, закрыть' }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

const STORAGE_KEY = 'tli_welcomed_v1';

const visible   = ref(false);
const countdown = ref(3);

let timer: ReturnType<typeof setInterval> | null = null;

function close() {
  if (countdown.value > 0) return;
  visible.value = false;
  localStorage.setItem(STORAGE_KEY, '1');
}

onMounted(() => {
  if (!localStorage.getItem(STORAGE_KEY)) {
    visible.value = true;
    timer = setInterval(() => {
      countdown.value--;
      if (countdown.value <= 0 && timer) {
        clearInterval(timer);
        timer = null;
      }
    }, 1000);
  }
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<style scoped>
.hotkey {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 40px;
  padding: 3px 10px;
  border-radius: 7px;
  font-size: 12px;
  font-weight: 800;
  font-family: 'Segoe UI', monospace;
  color: #60a5fa;
  background: rgba(96,165,250,0.12);
  border: 1px solid rgba(96,165,250,0.35);
  flex-shrink: 0;
}

.welcome-fade-enter-active { transition: all 0.3s ease-out; }
.welcome-fade-leave-active { transition: all 0.25s ease-in; }
.welcome-fade-enter-from   { opacity: 0; transform: scale(0.96); }
.welcome-fade-leave-to     { opacity: 0; transform: scale(0.96); }
</style>
