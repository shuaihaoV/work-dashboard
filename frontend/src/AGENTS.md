# frontend/src — React 前端

## OVERVIEW

React 18 + TypeScript (Vite 7) + TanStack Query 5 + TailwindCSS 3 + shadcn/ui，单文件 App.tsx 734 行含全部页面组件。

## STRUCTURE

```
frontend/src/
├── main.tsx                              # React 渲染入口
├── App.tsx                               # [非标准] 主组件，734 行含全部页面
├── api.ts                                # API 请求封装 + 全部 TS 类型定义
├── format.ts                             # 数值格式化工具（纯函数）
├── theme.tsx                             # ThemeProvider (system/light/dark)
├── styles.css                            # Tailwind 入口 + CSS 变量（亮/暗）
├── vite-env.d.ts                         # Vite 环境类型声明
├── components/
│   ├── date-time-range-picker.tsx        # 自定义时间范围选择器 (252 行)
│   ├── filter-combobox.tsx               # 用户/模型/渠道筛选组合框 (114 行)
│   ├── theme-toggle.tsx                  # 主题切换按钮
│   └── ui/                               # shadcn/ui 基础组件（源码复制，非 npm）
│       ├── badge.tsx, button.tsx, card.tsx
│       ├── command.tsx, popover.tsx
│       └── table.tsx, tabs.tsx
└── lib/
    └── utils.ts                          # cn() Tailwind 类合并
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| 修改页面 UI | `App.tsx` | 单文件，所有页面组件 + 筛选逻辑 + 6 个 Query |
| 添加 API 调用 | `api.ts` + 后端 `models.rs` | 双向同步，camelCase 对齐 |
| 添加 TS 类型 | `api.ts` | 所有 API 类型集中于此 |
| 数值格式化 | `format.ts` | 纯函数，`formatInt/TokenCompact/Percent/Ms/Tps/CacheRate/Quota` |
| 主题系统 | `theme.tsx` | `ThemeProvider` + `useTheme`，localStorage 持久化 |
| 时间范围选择器 | `components/date-time-range-picker.tsx` | 预设 + 自定义精确到分钟 |
| 筛选下拉 | `components/filter-combobox.tsx` | cmdk + Popover |
| 主题切换按钮 | `components/theme-toggle.tsx` | 三态循环 system→light→dark |
| shadcn 组件 | `components/ui/*.tsx` | 由 `components.json` 配置，源码复制 |
| 类名合并 | `lib/utils.ts` | `cn()` = `clsx + tailwind-merge` |
| 全局样式 | `styles.css` | Tailwind 指令 + `:root`/`.dark` CSS 变量 |

## CODE MAP

### main.tsx (27 行)
- `queryClient` :9 — `retry: 1, refetchOnWindowFocus: false, staleTime: 15_000`
- 渲染链：`StrictMode → ThemeProvider → QueryClientProvider → App`

### App.tsx (734 行 — 单体)
- 筛选状态：`dateRange, userId, modelId, channelId`
- 6 个 TanStack Query：`overview, users, channels, models, raw-models, extra`
- queryKey 包含全部 5 个筛选维度
- 自动刷新：时间范围含未来时 `refetchInterval = 60000`
- 预设时间：今天/本周/本月/过去7天/过去30天 + 自定义
- UI 区域：概览卡片、用户图表(recharts)、5 张统计表、Top 排行

### api.ts (190 行)
- `DateTimeRangeQuery` :1 — `{from, to, userId?, modelId?, channelId?}`
- `getBasePath()` :9 — 读 `window.__WORK_DASHBOARD_BASE_PATH__`
- `withBasePath()` :17 — 拼接子路径前缀
- `ApiResponse<T>` :21 — `{generatedAt, cacheTtlSec, data}`
- `request<T>()` :118 — 通用 GET 请求，URLSearchParams 构造查询
- `fetchOverview/UserStats/ChannelStats/ModelStats/RawModelStats/ExtraStats` :140-162 — 6 个统计 API
- `fetchUserOptions/ModelOptions/ChannelOptions` :164-174 — 3 个下拉搜索 API
- **所有 TS interface 与后端 `models.rs` 一一对应**（camelCase）

### format.ts (61 行 — 纯函数)
- `formatInt` :1 — `Intl.NumberFormat('en-US')`
- `formatTokenCompact` :5 — K/M/B/T 紧凑表示
- `formatPercent` :24 — `toFixed(2)%`
- `formatMs` :28 — `ms/s` 自适应，null → `-`
- `formatTps` :38 — `tok/s`
- `formatCacheRate` :42 — `cachedTokens/inputTokens * 100`
- `formatQuota` :49 — `QUOTA_PER_UNIT = 500_000`，美元换算 `$/$K/$M`

### theme.tsx (89 行)
- `ThemeMode` :11 — `'system' | 'light' | 'dark'`
- `ThemeProvider` :24 — useEffect 读 localStorage + matchMedia(`prefers-color-scheme`)
- `cycleMode` :59 — system→light→dark→system 循环
- `useTheme()` :83 — Context consumer，未在 Provider 内抛错
- `THEME_STORAGE_KEY = 'work-dashboard-theme-mode'`

### components/date-time-range-picker.tsx (252 行)
- 预设快捷项 + 自定义精确到分钟
- 输出 RFC3339 格式

### components/filter-combobox.tsx (114 行)
- 基于 cmdk + Popover 的搜索下拉
- 调用 `fetchUserOptions/ModelOptions/ChannelOptions`

### components/ui/*.tsx
- shadcn/ui 源码复制组件，由 `components.json` 配置
- 修改这些文件无意义，应通过 `npx shadcn-ui@latest add` 重新添加

### lib/utils.ts
- `cn()` — `clsx` + `tailwind-merge` 合并 Tailwind 类

## CONVENTIONS (frontend/src 特有)

- **路径别名**：`@/` → `frontend/src/`（vite.config.ts + tsconfig.app.json 双重配置）
- **API 类型集中**：所有 interface 定义在 `api.ts`，与后端 `models.rs` camelCase 对齐
- **格式化集中**：所有数值格式化在 `format.ts`，组件不内联格式化逻辑
- **TanStack Query**：queryKey = `[key, from, to, userId, modelId, channelId]`，含未来时自动刷新
- **主题三态**：system/light/dark 循环切换，非二态 toggle
- **中文 UI 文案**：预设时间、表头、按钮等全部中文
- **shadcn/ui 源码复制**：`components/ui/` 是源码非依赖，由 `components.json` 管理
- **CSS 变量主题**：`:root` 亮色变量 + `.dark` 暗色变量，无 JS 主题逻辑
- **fetch 非 axios**：使用原生 `fetch`，无 HTTP 客户端库
- **StrictMode**：开发模式启用，注意副作用可能执行两次

## ANTI-PATTERNS

- ❌ `any` 类型 / `as any` / `@ts-ignore` / `@ts-expect-error`
- ❌ 在组件中内联数值格式化逻辑（统一走 `format.ts`）
- ❌ 在组件中内联 API 类型定义（统一在 `api.ts`）
- ❌ 从非 `@/` 别名导入（如 `../../components/...`）
- ❌ 拆分 App.tsx 到多文件（项目刻意保持单体，页面数量少）
- ❌ 使用 axios 或其他 HTTP 客户端（统一用原生 `fetch`）
- ❌ 修改 `components/ui/*.tsx`（应通过 shadcn CLI 重新生成）
