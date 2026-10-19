<template>
  <div
    v-show="isVisible"
    ref="widgetRef"
    class="absolute inline-block max-w-[800px]"
    :class="[
      (isEditMode || props.alwaysInteractive) ? 'pointer-events-auto' : 'pointer-events-none',
      isStandalone ? '!static !block w-screen h-screen' : '',
    ]"
    v-bind="props.alwaysInteractive ? { 'data-interactive': '' } : {}"
    :style="
      isStandalone
        ? {}
        : {
            left: `${x}px`,
            top: `${y}px`,
            zIndex: isEditMode
              ? isDragging || isResizing
                ? 999999
                : currentZ
              : 10,
          }
    "
    @mousedown="onMouseDown"
  >
    <div
      v-show="isEditMode && !hideTitleBar"
      v-bind="isStandalone ? { 'data-tauri-drag-region': '' } : {}"
      class="bg-[#0078d7]/90 text-white text-[10px] uppercase font-bold px-2 py-[4px] flex items-center gap-2 select-none border-b-2 border-[#1E1E1E] absolute"
      :class="[
        isStandalone
          ? 'cursor-move w-screen h-[24px] pointer-events-auto top-0 left-0'
          : 'bottom-full left-0 rounded-t-md cursor-move min-w-[100px]',
      ]"
    >
      <svg
        class="w-[14px] h-[14px] opacity-70 hover:opacity-100 cursor-pointer z-50 relative pointer-events-auto"
        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
        aria-hidden="true"
        @click.stop="popOutWidget"
        :title="isStandalone ? 'Прикрепить обратно' : 'Открепить'"
      >
        <path v-if="isStandalone" d="M3 9H9V3M21 15H15V21M3 3 9 9M21 21 15 15" />
        <path v-else d="M14 3H21V10M21 3 11 13M10 5H4V20H19V14" />
      </svg>
      <span
        v-bind="isStandalone ? { 'data-tauri-drag-region': '' } : {}"
        class="flex-1 min-w-[100px] whitespace-nowrap"
        >{{ title }}</span
      >
    </div>

    <div
      class="select-none transition-shadow duration-200 overflow-hidden relative"
      :class="[
        noBackground ? '' : 'px-3 py-2 min-w-[100px] min-h-[40px]',
        isEditMode
          ? 'shadow-[0_0_15px_rgba(0,120,215,0.3)] border-2'
          : noBackground ? '' : 'border'
      ]"
      :style="{
        marginTop: (isStandalone && isEditMode && !hideTitleBar) ? '24px' : '0',
        width: isStandalone ? '100vw' : w ? `${w}px` : props.defaultW ? `${props.defaultW}px` : 'auto',
        height: isStandalone
          ? isEditMode
            ? 'calc(100vh - 24px)'
            : '100vh'
          : h
            ? `${h}px`
            : props.defaultH
              ? `${props.defaultH}px`
              : 'auto',
        '--widget-w': w || props.defaultW || 100,
        '--widget-h': h || props.defaultH || 60,
        backgroundColor: 'transparent',
        borderColor: noBackground ? 'transparent' : (isEditMode
          ? 'var(--theme-text-price)'
          : 'var(--theme-border-color)'),
        borderRadius:
          isEditMode && !isStandalone && !hideTitleBar
            ? '0 var(--theme-radius-widget) var(--theme-radius-widget) var(--theme-radius-widget)'
            : isStandalone
              ? '0'
              : 'var(--theme-radius-widget)',
      }"
    >
      <div
        v-if="!noBackground"
        class="absolute inset-0 pointer-events-none"
        :style="{
          backgroundColor: 'var(--theme-bg-widget)',
          opacity: store.widgetOpacity,
          borderRadius: 'inherit',
        }"
      />
      <div class="relative z-[1]">
        <slot></slot>
      </div>
    </div>

    <div
      v-show="isEditMode"
      @mousedown.stop.prevent="startResize"
      class="absolute bottom-0 right-0 w-3 h-3 bg-[#0078d7] rounded-tl-sm rounded-br-md cursor-nwse-resize opacity-50 hover:opacity-100"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useTrackerStore } from "../stores/trackerStore";
import {
  WebviewWindow,
  getCurrentWebviewWindow,
} from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";

const props = defineProps({
  widgetId: { type: String, required: true },
  title: { type: String, required: true },
  hideTitleBar: { type: Boolean, default: false },
  noBackground: { type: Boolean, default: false },
  defaultPosition: { type: Object, default: () => ({ x: 0, y: 0 }) },
  defaultW: { type: Number, default: undefined },
  defaultH: { type: Number, default: undefined },
  alwaysInteractive: { type: Boolean, default: false },
});

const store = useTrackerStore();
const widgetRef = ref<HTMLElement | null>(null);

const x = ref(0);
const y = ref(0);
const w = ref<number | undefined>(undefined);
const h = ref<number | undefined>(undefined);
const isDragging = ref(false);
const isResizing = ref(false);
const currentZ = ref(50);

const isEditMode = computed(() => store.isEditMode);
const isVisible  = computed(() => {
  const pos = (store.widgetPositions as any)[props.widgetId];
  return pos?.show !== false;
});

let startMouseX = 0;
let startMouseY = 0;
let initialX = 0;
let initialY = 0;
let initialW = 0;
let initialH = 0;

const SNAP_THRESHOLD = 15;

const queryStr =
  window.location.search || window.location.hash.split("?")[1] || "";
const isStandalone = !!new URLSearchParams(queryStr).get("widgetOnly");

const popOutWidget = async () => {
  if (isStandalone) {
    store.toggleWidget(props.widgetId as any);
    emit("restore-widget", props.widgetId);
    getCurrentWebviewWindow()
      .close()
      .catch((err) => {
        console.error("Failed to close", err);
        window.close();
      });
    return;
  }

  store.toggleWidget(props.widgetId as any);

  const windowLabel = `widget_${props.widgetId}_${Date.now()}`;
  const webview = new WebviewWindow(windowLabel, {
    url: `/#/?widgetOnly=${props.widgetId}&editMode=${store.isEditMode}`,
    title: props.title,
    width: w.value || 300,
    height: (h.value || 200) + 16,
    transparent: true,
    decorations: false,
    alwaysOnTop: true,
    dragDropEnabled: false,
  });

  webview.once("tauri://error", function (e: any) {
    console.error("Error creating popout window", e);
  });
};

const startResize = (event: MouseEvent) => {
  if (!isEditMode.value) return;

  if (isStandalone) {
    getCurrentWebviewWindow()
      .startResizeDragging("SouthEast")
      .catch((err) => console.error(err));
    return;
  }
  isResizing.value = true;
  startMouseX = event.clientX;
  startMouseY = event.clientY;

  initialW = w.value || widgetRef.value?.offsetWidth || 100;
  initialH = h.value || widgetRef.value?.offsetHeight || 40;

  window.addEventListener("mousemove", onResizeDrag);
  window.addEventListener("mouseup", stopResize);
};

const onResizeDrag = (event: MouseEvent) => {
  if (!isResizing.value) return;

  const newW = Math.max(100, initialW + (event.clientX - startMouseX));
  const newH = Math.max(40, initialH + (event.clientY - startMouseY));

  w.value = newW;
  h.value = newH;
};

const stopResize = () => {
  if (!isResizing.value) return;
  isResizing.value = false;
  window.removeEventListener("mousemove", onResizeDrag);
  window.removeEventListener("mouseup", stopResize);

  store.updatePosition(
    props.widgetId as any,
    x.value,
    y.value,
    w.value,
    h.value,
  );
  store.saveConfigToStorage();
};

const onMouseDown = (event: MouseEvent) => {
  if (!isEditMode.value) return;

  if (isStandalone) {
    let target = event.target as HTMLElement;
    if (
      target.hasAttribute("data-tauri-drag-region") ||
      target.closest("[data-tauri-drag-region]")
    ) {
      getCurrentWebviewWindow()
        .startDragging()
        .catch((err) => console.error(err));
    }
    return;
  }

  currentZ.value = Math.floor(Date.now() / 1000) % 100000;

  let target = event.target as HTMLElement;
  if (
    target.tagName.toLowerCase() === "button" ||
    target.tagName.toLowerCase() === "input"
  ) {
    return;
  }

  isDragging.value = true;
  startMouseX = event.clientX;
  startMouseY = event.clientY;
  initialX = x.value;
  initialY = y.value;

  window.addEventListener("mousemove", onDrag);
  window.addEventListener("mouseup", stopDrag);
};

const onDrag = (event: MouseEvent) => {
  if (!isDragging.value) return;

  let newX = initialX + (event.clientX - startMouseX);
  let newY = Math.max(0, initialY + (event.clientY - startMouseY));

  if (widgetRef.value) {
    const w = widgetRef.value.offsetWidth;
    const h = widgetRef.value.offsetHeight;

    const others = Object.entries(store.widgetPositions).filter(
      ([id, pos]) => id !== props.widgetId && pos.show,
    );

    let snappedX = false;
    let snappedY = false;

    for (const [_, pos] of others) {
      if (!pos.w || !pos.h) continue;

      const targetEdgesX = [pos.x, pos.x + pos.w];
      const myEdgesX = [newX, newX + w];

      for (const tEdge of targetEdgesX) {
        if (!snappedX && Math.abs(myEdgesX[0] - tEdge) < SNAP_THRESHOLD) {
          newX = tEdge;
          snappedX = true;
        }
        if (!snappedX && Math.abs(myEdgesX[1] - tEdge) < SNAP_THRESHOLD) {
          newX = tEdge - w;
          snappedX = true;
        }
      }

      const targetEdgesY = [pos.y, pos.y + pos.h];
      const myEdgesY = [newY, newY + h];

      for (const tEdge of targetEdgesY) {
        if (!snappedY && Math.abs(myEdgesY[0] - tEdge) < SNAP_THRESHOLD) {
          newY = tEdge;
          snappedY = true;
        }
        if (!snappedY && Math.abs(myEdgesY[1] - tEdge) < SNAP_THRESHOLD) {
          newY = tEdge - h;
          snappedY = true;
        }
      }
    }

    if (!snappedX && newX < SNAP_THRESHOLD) newX = 0;
    if (!snappedX && Math.abs(newX + w - window.innerWidth) < SNAP_THRESHOLD)
      newX = window.innerWidth - w;
    if (!snappedY && newY < SNAP_THRESHOLD) newY = 0;
    if (!snappedY && Math.abs(newY + h - window.innerHeight) < SNAP_THRESHOLD)
      newY = window.innerHeight - h;
  }

  x.value = newX;
  y.value = newY;
};

const stopDrag = () => {
  if (!isDragging.value) return;
  isDragging.value = false;
  window.removeEventListener("mousemove", onDrag);
  window.removeEventListener("mouseup", stopDrag);

  const w = widgetRef.value?.offsetWidth;
  const h = widgetRef.value?.offsetHeight;
  store.updatePosition(props.widgetId as any, x.value, y.value, w, h);
  store.saveConfigToStorage();
};

onMounted(() => {
  setTimeout(() => {
    let saved = (store.widgetPositions as any)[props.widgetId];
    if (saved && saved.x !== undefined) {
      x.value = saved.x;
      y.value = saved.y;
      if (saved.w) w.value = saved.w;
      if (saved.h) h.value = saved.h;
      } else if (props.defaultPosition) {
        x.value = props.defaultPosition.x;
        y.value = props.defaultPosition.y;
      }

      if (!w.value || !h.value) {
        const wDom = widgetRef.value?.offsetWidth;
        const hDom = widgetRef.value?.offsetHeight;
        if (wDom && hDom) {
          if (!w.value) w.value = wDom;
          if (!h.value) h.value = hDom;
          store.updatePosition(
            props.widgetId as any,
            x.value,
            y.value,
            wDom,
            hDom,
          );
        }
      }
    }, 100);
  });

watch(
  () => (store.widgetPositions as any)[props.widgetId],
  (newPos) => {
    if (newPos) {
      x.value = newPos.x;
      y.value = newPos.y;
      if (newPos.w) w.value = newPos.w;
      if (newPos.h) h.value = newPos.h;
    }
  },
  { deep: true },
);
</script>
