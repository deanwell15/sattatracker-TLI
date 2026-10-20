<template>
  <img v-if="!failed" :src="source" :alt="alt" @error="onError" />
  <span v-else data-asset-fallback role="img" :aria-label="alt || 'Graphic unavailable'"
    class="inline-flex items-center justify-center rounded bg-slate-800 text-amber-300"
  >◇</span>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { getIcon } from '../utils/assets';

const props = withDefaults(defineProps<{ src?: string; alt?: string }>(), { alt: '' });
const fallback = getIcon('');
const source = ref(props.src || fallback);
const failed = ref(false);

watch(() => props.src, (src) => {
  source.value = src || fallback;
  failed.value = false;
});

function onError() {
  if (source.value !== fallback) {
    source.value = fallback;
  } else {
    // Terminal CSS fallback: no retry loop even if the bundled image is broken.
    failed.value = true;
  }
}
</script>
