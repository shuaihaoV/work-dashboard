export interface DateTimeRangeQuery {
  from: string;
  to: string;
  userId?: number[] | null;
  modelId?: string[] | null;
  channelId?: number[] | null;
  tokenName?: string[] | null;
  group?: string[] | null;
}

function getBasePath(): string {
  const configured = window.__WORK_DASHBOARD_BASE_PATH__;
  if (!configured || configured === '/') {
    return '';
  }
  return configured.endsWith('/') ? configured.slice(0, -1) : configured;
}

function withBasePath(path: string): string {
  return `${getBasePath()}${path}`;
}

export interface ApiResponse<T> {
  generatedAt: string;
  cacheTtlSec: number;
  data: T;
}

export interface OverviewStats {
  totalRequests: number;
  successRate: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCachedTokens: number;
  totalQuota: number;
  avgLatencyMs: number | null;
  avgFrtMs: number | null;
}

export interface UserStatsItem {
  userId: number;
  userName: string;
  totalRequests: number;
  successRate: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  quotaUsed: number;
  avgLatencyMs: number | null;
}

export interface UserOptionItem {
  userId: number;
  userName: string;
}

export interface ModelOptionItem {
  modelName: string;
}

export interface ChannelOptionItem {
  channelId: number;
  channelName: string;
}

export interface ChannelStatsItem {
  channelId: number;
  channelName: string;
  channelType: string;
  status: string;
  totalRequests: number;
  successRate: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  quotaUsed: number;
  avgLatencyMs: number | null;
}

export interface ModelStatsItem {
  modelName: string;
  totalRequests: number;
  successRate: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  quotaUsed: number;
  avgLatencyMs: number | null;
}

export interface RawModelStatsItem {
  modelName: string;
  channelId: number;
  channelName: string;
  channelType: string;
  totalRequests: number;
  successRate: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  quotaUsed: number;
  avgLatencyMs: number | null;
}

export interface TopThroughputChannel {
  channelId: number;
  channelName: string;
  tokensPerSecond: number;
  requestCount: number;
}

export interface TopRequestedModel {
  modelName: string;
  totalRequests: number;
}

export interface ExtraStats {
  topThroughputChannels: TopThroughputChannel[];
  topRequestedModels: TopRequestedModel[];
}

export interface TimeseriesPoint {
  bucketTs: number;
  requestCount: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  avgLatencyMs: number | null;
}

export interface TokenStatsItem {
  tokenName: string;
  totalRequests: number;
  successRate: number;
  inputTokens: number;
  outputTokens: number;
  cachedTokens: number;
  avgLatencyMs: number | null;
}

export interface TokenOptionItem {
  tokenName: string;
}

export interface PerfMetricStats {
  modelName: string;
  requestCount: number;
  successRate: number;
  avgLatencyMs: number | null;
  avgTtftMs: number | null;
  outputTokens: number;
  generationSpeedTps: number | null;
}

async function request<T>(path: string, range: DateTimeRangeQuery): Promise<ApiResponse<T>> {
  const params = new URLSearchParams({
    from: range.from,
    to: range.to,
  });
  if (range.userId && range.userId.length > 0) {
    for (const id of range.userId) {
      params.append('userId[]', String(id));
    }
  }
  if (range.modelId && range.modelId.length > 0) {
    for (const name of range.modelId) {
      params.append('modelId[]', name);
    }
  }
  if (range.channelId && range.channelId.length > 0) {
    for (const id of range.channelId) {
      params.append('channelId[]', String(id));
    }
  }
  if (range.tokenName && range.tokenName.length > 0) {
    for (const name of range.tokenName) {
      params.append('tokenName[]', name);
    }
  }
  if (range.group && range.group.length > 0) {
    for (const name of range.group) {
      params.append('group[]', name);
    }
  }
  const response = await fetch(`${withBasePath(path)}?${params.toString()}`);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed (${response.status}): ${text}`);
  }
  return (await response.json()) as ApiResponse<T>;
}

export function fetchOverview(range: DateTimeRangeQuery) {
  return request<OverviewStats>('/api/v1/overview', range);
}

export function fetchUserStats(range: DateTimeRangeQuery) {
  return request<UserStatsItem[]>('/api/v1/stats/users', range);
}

export function fetchChannelStats(range: DateTimeRangeQuery) {
  return request<ChannelStatsItem[]>('/api/v1/stats/channels', range);
}

export function fetchModelStats(range: DateTimeRangeQuery) {
  return request<ModelStatsItem[]>('/api/v1/stats/models', range);
}

export function fetchRawModelStats(range: DateTimeRangeQuery) {
  return request<RawModelStatsItem[]>('/api/v1/stats/raw-models', range);
}

export function fetchExtraStats(range: DateTimeRangeQuery) {
  return request<ExtraStats>('/api/v1/stats/extra', range);
}

export async function fetchUserOptions(keyword: string): Promise<ApiResponse<UserOptionItem[]>> {
  return fetchLookupOptions<UserOptionItem>('/api/v1/users/search', keyword);
}

export async function fetchModelOptions(keyword: string): Promise<ApiResponse<ModelOptionItem[]>> {
  return fetchLookupOptions<ModelOptionItem>('/api/v1/models/search', keyword);
}

export async function fetchChannelOptions(keyword: string): Promise<ApiResponse<ChannelOptionItem[]>> {
  return fetchLookupOptions<ChannelOptionItem>('/api/v1/channels/search', keyword);
}

async function fetchLookupOptions<T>(path: string, keyword: string): Promise<ApiResponse<T[]>> {
  const params = new URLSearchParams();
  const normalized = keyword.trim();
  if (normalized) {
    params.set('q', normalized);
  }

  const fullPath = params.size > 0 ? `${path}?${params.toString()}` : path;
  const response = await fetch(withBasePath(fullPath));
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed (${response.status}): ${text}`);
  }
  return (await response.json()) as ApiResponse<T[]>;
}

export async function fetchTimeseries(
  range: DateTimeRangeQuery,
  granularity: 'hour' | 'day',
): Promise<ApiResponse<TimeseriesPoint[]>> {
  const params = new URLSearchParams({
    from: range.from,
    to: range.to,
    granularity,
  });
  if (range.userId && range.userId.length > 0) {
    for (const id of range.userId) {
      params.append('userId[]', String(id));
    }
  }
  if (range.modelId && range.modelId.length > 0) {
    for (const name of range.modelId) {
      params.append('modelId[]', name);
    }
  }
  if (range.channelId && range.channelId.length > 0) {
    for (const id of range.channelId) {
      params.append('channelId[]', String(id));
    }
  }
  if (range.tokenName && range.tokenName.length > 0) {
    for (const name of range.tokenName) {
      params.append('tokenName[]', name);
    }
  }
  if (range.group && range.group.length > 0) {
    for (const name of range.group) {
      params.append('group[]', name);
    }
  }
  const response = await fetch(`${withBasePath('/api/v1/stats/timeseries')}?${params.toString()}`);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed (${response.status}): ${text}`);
  }
  return (await response.json()) as ApiResponse<TimeseriesPoint[]>;
}

export function fetchTokenStats(range: DateTimeRangeQuery) {
  return request<TokenStatsItem[]>('/api/v1/stats/tokens', range);
}

export async function fetchTokenOptions(keyword: string): Promise<ApiResponse<TokenOptionItem[]>> {
  return fetchLookupOptions<TokenOptionItem>('/api/v1/tokens/search', keyword);
}

export function fetchPerfMetrics(range: DateTimeRangeQuery) {
  return request<PerfMetricStats[]>('/api/v1/stats/perf', range);
}
