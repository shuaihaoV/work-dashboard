export function formatInt(value: number): string {
  return new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 }).format(value);
}

export function formatTokenCompact(value: number): string {
  const abs = Math.abs(value);

  if (abs >= 1_000_000_000_000) {
    return `${(value / 1_000_000_000_000).toFixed(2)}T`;
  }
  if (abs >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(2)}B`;
  }
  if (abs >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(2)}M`;
  }
  if (abs >= 1_000) {
    return `${(value / 1_000).toFixed(2)}K`;
  }

  return value.toFixed(2);
}

export function formatPercent(value: number): string {
  return `${value.toFixed(2)}%`;
}

export function formatMs(value: number | null): string {
  if (value === null || Number.isNaN(value)) {
    return '-';
  }
  if (Math.abs(value) >= 1000) {
    return `${(value / 1000).toFixed(1)} s`;
  }
  return `${value.toFixed(1)} ms`;
}

export function formatTps(value: number): string {
  return `${value.toFixed(2)} tok/s`;
}

export function formatCacheRate(cachedTokens: number, inputTokens: number): string {
  if (inputTokens <= 0) return '0.00%';
  return `${((cachedTokens / inputTokens) * 100).toFixed(2)}%`;
}

const QUOTA_PER_UNIT = 500_000;

export function formatQuota(value: number): string {
  const dollars = value / QUOTA_PER_UNIT;
  if (dollars >= 1000) {
    return `$${(dollars / 1000).toFixed(2)}K`;
  }
  if (dollars >= 1) {
    return `$${dollars.toFixed(2)}`;
  }
  if (dollars >= 0.01) {
    return `$${dollars.toFixed(4)}`;
  }
  return `$${dollars.toFixed(6)}`;
}
