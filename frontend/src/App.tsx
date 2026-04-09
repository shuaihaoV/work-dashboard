import { useMemo, useState, type ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Activity, Database, Gauge, Sparkles, Users } from 'lucide-react';

import {
  fetchChannelOptions,
  fetchChannelStats,
  fetchExtraStats,
  fetchModelOptions,
  fetchModelStats,
  fetchOverview,
  fetchUserOptions,
  fetchUserStats,
  type ChannelOptionItem,
  type UserOptionItem,
  type ModelOptionItem,
  fetchRawModelStats,
  type ApiResponse,
  type ChannelStatsItem,
  type ExtraStats,
  type ModelStatsItem,
  type OverviewStats,
  type UserStatsItem,
  type RawModelStatsItem,
} from '@/api';
import { DateTimeRangePicker, defaultRangeToday, type DateTimeRange } from '@/components/date-time-range-picker';
import { FilterCombobox } from '@/components/filter-combobox';
import { ThemeToggle } from '@/components/theme-toggle';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { formatCacheRate, formatInt, formatMs, formatPercent, formatTokenCompact, formatTps } from '@/format';

const REFRESH_MS = 60_000;

function App() {
  const [range, setRange] = useState<DateTimeRange>(defaultRangeToday());
  const [selectedUser, setSelectedUser] = useState<UserOptionItem | null>(null);
  const [selectedModel, setSelectedModel] = useState<ModelOptionItem | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<ChannelOptionItem | null>(null);
  const refetchInterval = range.to.getTime() > Date.now() ? REFRESH_MS : false;

  const queryRange = useMemo(
    () => ({
      from: range.from.toISOString(),
      to: range.to.toISOString(),
      userId: selectedUser?.userId ?? null,
      modelId: selectedModel?.modelName ?? null,
      channelId: selectedChannel?.channelId ?? null,
    }),
    [
      range.from.getTime(),
      range.to.getTime(),
      selectedUser?.userId,
      selectedModel?.modelName,
      selectedChannel?.channelId,
    ]
  );

  const overviewQuery = useQuery({
    queryKey: [
      'overview',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchOverview(queryRange),
    refetchInterval,
  });

  const userQuery = useQuery({
    queryKey: [
      'users',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchUserStats(queryRange),
    refetchInterval,
  });

  const channelQuery = useQuery({
    queryKey: [
      'channels',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchChannelStats(queryRange),
    refetchInterval,
  });

  const modelQuery = useQuery({
    queryKey: [
      'models',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchModelStats(queryRange),
    refetchInterval,
  });

  const rawModelQuery = useQuery({
    queryKey: [
      'raw-models',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchRawModelStats(queryRange),
    refetchInterval,
  });

  const extraQuery = useQuery({
    queryKey: [
      'extra',
      queryRange.from,
      queryRange.to,
      queryRange.userId ?? 'all',
      queryRange.modelId ?? 'all',
      queryRange.channelId ?? 'all',
    ],
    queryFn: () => fetchExtraStats(queryRange),
    refetchInterval,
  });

  const errors = [
    overviewQuery.error,
    userQuery.error,
    channelQuery.error,
    modelQuery.error,
    rawModelQuery.error,
    extraQuery.error,
  ]
    .filter(Boolean)
    .map((err) => (err as Error).message);

  return (
    <div className="min-h-screen bg-[radial-gradient(1200px_500px_at_85%_-50%,#9ae6b422,transparent),linear-gradient(180deg,#f7fafc_0%,#edf2f7_100%)] text-foreground dark:bg-[radial-gradient(1200px_500px_at_85%_-50%,#22d3ee22,transparent),linear-gradient(180deg,#0f172a_0%,#020617_100%)]">
      <main className="mx-auto flex w-full max-w-none flex-col gap-4 px-2 py-3 md:px-3 md:py-4 lg:px-4">
        <header className="relative overflow-hidden rounded-[24px] border border-border/70 bg-card/82 p-3 shadow-[0_24px_80px_-32px_rgba(15,23,42,0.35)] backdrop-blur-xl md:p-4">
          <div className="absolute inset-0 bg-[linear-gradient(135deg,rgba(14,165,233,0.10),transparent_42%,rgba(16,185,129,0.08))]" />
          <div className="relative grid gap-2.5 grid-cols-2 sm:grid-cols-3 md:grid-cols-[44px_minmax(180px,1fr)_minmax(180px,1fr)_minmax(180px,1fr)_minmax(280px,1.1fr)] md:items-center">
            <div className="justify-self-start self-start md:self-center">
              <ThemeToggle />
            </div>

            <div>
              <FilterCombobox
                value={selectedUser}
                onChange={setSelectedUser}
                fetchOptions={fetchUserOptions}
                queryKeyPrefix="user-options"
                getOptionKey={(option) => option.userId}
                getOptionLabel={(option) => option.userName}
                allLabel="全部用户"
                searchPlaceholder="输入用户 ID 或名称搜索"
              />
            </div>

            <div>
              <FilterCombobox
                value={selectedModel}
                onChange={setSelectedModel}
                fetchOptions={fetchModelOptions}
                queryKeyPrefix="model-options"
                getOptionKey={(option) => option.modelName}
                getOptionLabel={(option) => option.modelName}
                allLabel="全部模型ID"
                searchPlaceholder="输入模型ID搜索"
              />
            </div>

            <div>
              <FilterCombobox
                value={selectedChannel}
                onChange={setSelectedChannel}
                fetchOptions={fetchChannelOptions}
                queryKeyPrefix="channel-options"
                getOptionKey={(option) => option.channelId}
                getOptionLabel={(option) => option.channelName}
                allLabel="全部渠道"
                searchPlaceholder="输入渠道 ID 或名称搜索"
              />
            </div>

            <div className="col-span-2 sm:col-span-2 sm:[grid-column:2/span_2] md:col-span-1 md:[grid-column:auto]">
              <DateTimeRangePicker
                value={range}
                onChange={setRange}
                className="w-full md:w-auto"
              />
            </div>
          </div>
        </header>

        {errors.length > 0 ? (
          <Card className="border-rose-300 bg-rose-50 dark:bg-rose-950/20">
            <CardHeader>
              <CardTitle>数据加载失败</CardTitle>
              <CardDescription>{errors[0]}</CardDescription>
            </CardHeader>
          </Card>
        ) : null}

        <OverviewSection data={overviewQuery.data} loading={overviewQuery.isLoading} />

        <StatsTableSection
          title="用户统计"
          description="按时间范围查看用户请求量、成功率和 Token 消耗"
          icon={<Users className="h-4 w-4" />}
          loading={userQuery.isLoading}
          rows={userQuery.data?.data ?? []}
          renderHead={() => (
            <>
              <TableHead>用户</TableHead>
              <TableHead className="text-right">请求数</TableHead>
              <TableHead className="text-right">输入 Token</TableHead>
              <TableHead className="text-right">输出 Token</TableHead>
              <TableHead className="text-right">缓存 Token</TableHead>
              <TableHead className="text-right">平均延迟</TableHead>
              <TableHead className="text-right">缓存率</TableHead>
              <TableHead className="text-right">成功率</TableHead>
            </>
          )}
          renderRow={(row) => <UserRow key={row.userId} row={row} />}
        />

        <StatsTableSection
          title="模型统计"
          description="按模型查看调用质量与 Token 消耗"
          icon={<Database className="h-4 w-4" />}
          loading={modelQuery.isLoading}
          rows={modelQuery.data?.data ?? []}
          renderHead={() => (
            <>
              <TableHead>模型</TableHead>
              <TableHead className="text-right">请求数</TableHead>
              <TableHead className="text-right">输入 Token</TableHead>
              <TableHead className="text-right">输出 Token</TableHead>
              <TableHead className="text-right">缓存 Token</TableHead>
              <TableHead className="text-right">平均延迟</TableHead>
              <TableHead className="text-right">缓存率</TableHead>
              <TableHead className="text-right">成功率</TableHead>
            </>
          )}
          renderRow={(row) => <ModelRow key={row.modelName} row={row} />}
        />

        <StatsTableSection
          title="渠道统计"
          description="按渠道观察启用状态、稳定性与 Token 消耗"
          icon={<Gauge className="h-4 w-4" />}
          loading={channelQuery.isLoading}
          rows={channelQuery.data?.data ?? []}
          renderHead={() => (
            <>
              <TableHead>渠道</TableHead>
              <TableHead>类型</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">请求数</TableHead>
              <TableHead className="text-right">输入 Token</TableHead>
              <TableHead className="text-right">输出 Token</TableHead>
              <TableHead className="text-right">缓存 Token</TableHead>
              <TableHead className="text-right">平均延迟</TableHead>
              <TableHead className="text-right">缓存率</TableHead>
              <TableHead className="text-right">成功率</TableHead>
            </>
          )}
          renderRow={(row) => <ChannelRow key={row.channelId} row={row} />}
        />

        <StatsTableSection
          title="原始模型统计"
          description="按模型与渠道交叉聚合（模型 / 渠道 / 类型 / 请求与延迟）"
          icon={<Database className="h-4 w-4" />}
          loading={rawModelQuery.isLoading}
          rows={rawModelQuery.data?.data ?? []}
          renderHead={() => (
            <>
              <TableHead>模型</TableHead>
              <TableHead>渠道</TableHead>
              <TableHead>类型</TableHead>
              <TableHead className="text-right">请求数</TableHead>
              <TableHead className="text-right">输入 Token</TableHead>
              <TableHead className="text-right">输出 Token</TableHead>
              <TableHead className="text-right">缓存 Token</TableHead>
              <TableHead className="text-right">平均延迟</TableHead>
              <TableHead className="text-right">缓存率</TableHead>
              <TableHead className="text-right">成功率</TableHead>
            </>
          )}
          renderRow={(row) => <RawModelRow key={`${row.modelName}-${row.channelId}`} row={row} />}
        />

        <ExtraSection data={extraQuery.data} loading={extraQuery.isLoading} />
      </main>
    </div>
  );
}

function OverviewSection({ data, loading }: { data?: ApiResponse<OverviewStats>; loading: boolean }) {
  const overview = data?.data;

  return (
    <section className="grid grid-cols-2 gap-3 lg:grid-cols-4">
      <MetricCard
        title="请求与成功率"
        value={
          loading && !overview
            ? '加载中...'
            : `${formatInt(overview?.totalRequests ?? 0)} / ${formatPercent(overview?.successRate ?? 0)}`
        }
        icon={<Activity className="h-4 w-4" />}
      />
      <MetricCard
        title="输入 Token"
        value={loading && !overview ? '加载中...' : formatTokenCompact(overview?.totalInputTokens ?? 0)}
        icon={<Database className="h-4 w-4" />}
      />
      <MetricCard
        title="缓存 Token"
        value={loading && !overview ? '加载中...' : formatTokenCompact(overview?.totalCachedTokens ?? 0)}
        icon={<Sparkles className="h-4 w-4" />}
      />
      <MetricCard
        title="输出 Token"
        value={loading && !overview ? '加载中...' : formatTokenCompact(overview?.totalOutputTokens ?? 0)}
        icon={<Gauge className="h-4 w-4" />}
      />
    </section>
  );
}

function MetricCard({ title, value, icon }: { title: string; value: string; icon?: ReactNode }) {
  return (
    <Card>
      <CardHeader className="pb-1.5">
        <CardDescription className="flex items-center justify-between text-[11px] uppercase tracking-wide">
          {title}
          {icon}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <p className="text-xl font-semibold tracking-tight md:text-2xl">{value}</p>
      </CardContent>
    </Card>
  );
}

function StatsTableSection<T>({
  title,
  description,
  icon,
  rows,
  loading,
  renderHead,
  renderRow,
}: {
  title: string;
  description: string;
  icon: ReactNode;
  rows: T[];
  loading: boolean;
  renderHead: () => ReactNode;
  renderRow: (row: T) => ReactNode;
}) {
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-lg md:text-xl">
          {icon} {title}
        </CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>{renderHead()}</TableRow>
          </TableHeader>
          <TableBody>
            {loading && rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={12} className="py-8 text-center text-muted-foreground">
                  加载中...
                </TableCell>
              </TableRow>
            ) : rows.length === 0 ? (
              <TableRow>
                <TableCell colSpan={12} className="py-8 text-center text-muted-foreground">
                  当前范围暂无数据
                </TableCell>
              </TableRow>
            ) : (
              rows.slice(0, 20).map((row) => renderRow(row))
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function UserRow({ row }: { row: UserStatsItem }) {
  return (
    <TableRow>
      <TableCell className="font-medium">{row.userName}</TableCell>
      <TableCell className="text-right">{formatInt(row.totalRequests)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.inputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.outputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.cachedTokens)}</TableCell>
      <TableCell className="text-right">{formatMs(row.avgLatencyMs)}</TableCell>
      <TableCell className="text-right">{formatCacheRate(row.cachedTokens, row.inputTokens)}</TableCell>
      <TableCell className="text-right">
        <RateBadge value={row.successRate} />
      </TableCell>
    </TableRow>
  );
}

function ChannelRow({ row }: { row: ChannelStatsItem }) {
  return (
    <TableRow>
      <TableCell className="font-medium">{row.channelName}</TableCell>
      <TableCell>
        <Badge variant="outline">{row.channelType}</Badge>
      </TableCell>
      <TableCell>
        <StatusBadge status={row.status} />
      </TableCell>
      <TableCell className="text-right">{formatInt(row.totalRequests)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.inputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.outputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.cachedTokens)}</TableCell>
      <TableCell className="text-right">{formatMs(row.avgLatencyMs)}</TableCell>
      <TableCell className="text-right">{formatCacheRate(row.cachedTokens, row.inputTokens)}</TableCell>
      <TableCell className="text-right">
        <RateBadge value={row.successRate} />
      </TableCell>
    </TableRow>
  );
}

function ModelRow({ row }: { row: ModelStatsItem }) {
  return (
    <TableRow>
      <TableCell className="font-medium">{row.modelName}</TableCell>
      <TableCell className="text-right">{formatInt(row.totalRequests)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.inputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.outputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.cachedTokens)}</TableCell>
      <TableCell className="text-right">{formatMs(row.avgLatencyMs)}</TableCell>
      <TableCell className="text-right">{formatCacheRate(row.cachedTokens, row.inputTokens)}</TableCell>
      <TableCell className="text-right">
        <RateBadge value={row.successRate} />
      </TableCell>
    </TableRow>
  );
}

function RawModelRow({ row }: { row: RawModelStatsItem }) {
  return (
    <TableRow>
      <TableCell className="font-medium">{row.modelName}</TableCell>
      <TableCell>{row.channelName}</TableCell>
      <TableCell>
        <Badge variant="outline">{row.channelType}</Badge>
      </TableCell>
      <TableCell className="text-right">{formatInt(row.totalRequests)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.inputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.outputTokens)}</TableCell>
      <TableCell className="text-right">{formatTokenCompact(row.cachedTokens)}</TableCell>
      <TableCell className="text-right">{formatMs(row.avgLatencyMs)}</TableCell>
      <TableCell className="text-right">{formatCacheRate(row.cachedTokens, row.inputTokens)}</TableCell>
      <TableCell className="text-right">
        <RateBadge value={row.successRate} />
      </TableCell>
    </TableRow>
  );
}

function RateBadge({ value }: { value: number }) {
  if (value >= 98) {
    return <Badge variant="success">{formatPercent(value)}</Badge>;
  }
  if (value >= 90) {
    return <Badge variant="warning">{formatPercent(value)}</Badge>;
  }
  return <Badge variant="danger">{formatPercent(value)}</Badge>;
}

function StatusBadge({ status }: { status: string }) {
  if (status === 'enabled') {
    return <Badge variant="success">启用</Badge>;
  }
  if (status === 'disabled') {
    return <Badge variant="warning">禁用</Badge>;
  }
  if (status === 'auto_disabled') {
    return <Badge variant="danger">自动禁用</Badge>;
  }
  return <Badge variant="outline">未知</Badge>;
}

function ExtraSection({ data, loading }: { data?: ApiResponse<ExtraStats>; loading: boolean }) {
  const extra = data?.data;

  return (
    <section className="grid grid-cols-1 gap-3 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>高吞吐渠道 Top</CardTitle>
          <CardDescription>按输出 Token / 总耗时（秒）估算吞吐，已过滤低样本渠道</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2">
          {loading && !extra ? <p className="text-sm text-muted-foreground">加载中...</p> : null}
          {!loading && (extra?.topThroughputChannels?.length ?? 0) === 0 ? (
            <p className="text-sm text-muted-foreground">当前范围暂无可用于吞吐排行的渠道数据</p>
          ) : null}
          {(extra?.topThroughputChannels ?? []).map((item) => (
            <div key={item.channelId} className="space-y-1 rounded-md border border-border/60 p-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="font-medium">{item.channelName}</span>
                <span>{formatTps(item.tokensPerSecond)}</span>
              </div>
              <p className="text-xs text-muted-foreground">{formatInt(item.requestCount)} 次请求样本</p>
            </div>
          ))}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>高频模型 Top</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {loading && !extra ? <p className="text-sm text-muted-foreground">加载中...</p> : null}
          {!loading && (extra?.topRequestedModels?.length ?? 0) === 0 ? (
            <p className="text-sm text-muted-foreground">当前范围暂无模型请求数据</p>
          ) : null}
          {(extra?.topRequestedModels ?? []).map((item) => (
            <div key={item.modelName} className="flex items-center justify-between text-sm">
              <span className="font-medium">{item.modelName}</span>
              <span>{formatInt(item.totalRequests)}</span>
            </div>
          ))}
        </CardContent>
      </Card>
    </section>
  );
}

export default App;
