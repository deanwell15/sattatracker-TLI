export type PriceStatus = 'known' | 'unknown' | 'stale';
export interface PriceQuote {
  value: number | null;
  status: PriceStatus;
  source: string | null;
  updated_at: string | null;
  received_at: string | null;
}
export interface PricingSnapshot {
  revision: number;
  season_id: number;
  source: 'automatic' | 'local_json';
  connection: string;
  last_success_at: string | null;
  error: string | null;
  available_count: number;
  cloud_count?: number;
  local_count?: number;
  refresh_seconds: number;
  titrack_enabled: boolean;
  next_retry_at?: string | null;
  prices: Record<string, PriceQuote>;
}
export const unknownPrice = (): PriceQuote => ({ value: null, status: 'unknown', source: null, updated_at: null, received_at: null });
export function priceLabel(item: { price: number; valuation?: PriceQuote }, quantity = 1, digits = 1): string {
  if (item.valuation?.status === 'unknown') return '—';
  return `${(item.price * quantity).toFixed(digits)}${item.valuation?.status === 'stale' ? ' ≈ stale' : ''}`;
}
export function priceDetails(item: { valuation?: PriceQuote }): string {
  const q = item.valuation;
  if (!q || q.status === 'unknown') return 'No market price available; quantity is still counted';
  return `${q.status}; ${q.source ?? 'unknown source'}; updated ${q.updated_at ?? 'base currency'}`;
}
