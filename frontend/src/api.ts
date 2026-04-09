export interface DateTimeRangeQuery {
  from: string;
  to: string;
  userId?: number | null;
  modelId?: string | null;
  channelId?: number | null;
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

async function request<T>(path: string, range: DateTimeRangeQuery): Promise<ApiResponse<T>> {
  const params = new URLSearchParams({
    from: range.from,
    to: range.to,
  });
  if (range.userId != null) {
    params.set('userId', String(range.userId));
  }
  if (range.modelId) {
    params.set('modelId', range.modelId);
  }
  if (range.channelId != null) {
    params.set('channelId', String(range.channelId));
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
