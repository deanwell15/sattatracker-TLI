import itemPlaceholder from '../assets/local/item.svg?url&no-inline';
import coinPlaceholder from '../assets/local/coin.svg?url&no-inline';
import effectPlaceholder from '../assets/local/effect.svg?url&no-inline';

// Stable call interfaces; no disk lookup, IPC, cache, ZIP or network provider.
// IDs are deliberately not interpreted: names and values remain separate data.
export function getIcon(_id: string | number): string {
  return itemPlaceholder;
}

export function getAnim(subpath: string): string {
  return subpath.startsWith('coin/') ? coinPlaceholder : effectPlaceholder;
}
