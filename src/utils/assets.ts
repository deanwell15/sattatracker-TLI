import itemPlaceholder from '../assets/local/item.svg?url&no-inline';
import { convertFileSrc } from '@tauri-apps/api/core';
import icons from '../assets/original-icons.json';

const iconIds = new Set(icons.ids);
const frames = import.meta.glob<string>('../assets/animations/**/*.png', {
  eager: true, query: '?url&no-inline', import: 'default',
});

// Unknown IDs and failed images alone use the neutral fallback.
export function getIcon(id: string | number): string {
  const key = String(id);
  if (!iconIds.has(key)) return itemPlaceholder;
  // This named custom protocol accepts only catalog icon names, never file paths.
  return `${convertFileSrc(`${key}.png`, 'deanfinite-icon')}?v=${icons.revision}`;
}

export function getAnim(subpath: string): string {
  const path = /^pig_\d+$/.test(subpath)
    ? `../assets/animations/pig/${subpath}.png`
    : /^coin\/coin_\d+$/.test(subpath)
      ? `../assets/animations/${subpath}.png`
      : '';
  return frames[path] ?? itemPlaceholder;
}
