import { addDays, addMonths, format, isValid, parse, startOfDay, startOfMonth, startOfWeek, subDays } from 'date-fns';
import { useEffect, useMemo, useState } from 'react';
import { CalendarRange } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { cn } from '@/lib/utils';

export interface DateTimeRange {
  from: Date;
  to: Date;
}

interface DateTimeRangePickerProps {
  value: DateTimeRange;
  onChange: (range: DateTimeRange) => void;
  className?: string;
}

type PresetKey = 'today' | 'thisWeek' | 'thisMonth' | 'last7days' | 'last30days' | 'custom';

const PRESET_LABELS: Record<Exclude<PresetKey, 'custom'>, string> = {
  today: '今天',
  thisWeek: '本周',
  thisMonth: '本月',
  last7days: '过去7天',
  last30days: '过去30天',
};

function toLocalInputValue(date: Date): string {
  return format(date, 'yyyy-MM-dd HH:mm');
}

function parseLocalInputValue(value: string): Date | null {
  if (!value) {
    return null;
  }
  const parsed = parse(value, 'yyyy-MM-dd HH:mm', new Date());
  if (!isValid(parsed)) {
    return null;
  }

  if (format(parsed, 'yyyy-MM-dd HH:mm') !== value) {
    return null;
  }

  return parsed;
}

function buildPresetRange(key: Exclude<PresetKey, 'custom'>): DateTimeRange {
  const now = new Date();
  switch (key) {
    case 'today': {
      return {
        from: startOfDay(now),
        to: startOfDay(addDays(now, 1)),
      };
    }
    case 'thisWeek': {
      const weekStart = startOfWeek(now, { weekStartsOn: 1 });
      return {
        from: weekStart,
        to: addDays(weekStart, 7),
      };
    }
    case 'thisMonth': {
      return {
        from: startOfMonth(now),
        to: startOfMonth(addMonths(now, 1)),
      };
    }
    case 'last7days':
      return {
        from: startOfDay(subDays(now, 6)),
        to: now,
      };
    case 'last30days':
      return {
        from: startOfDay(subDays(now, 29)),
        to: now,
      };
    default:
      return {
        from: startOfDay(now),
        to: startOfDay(addDays(now, 1)),
      };
  }
}

function sameMinute(a: Date, b: Date): boolean {
  return Math.floor(a.getTime() / 60_000) === Math.floor(b.getTime() / 60_000);
}

function detectPreset(range: DateTimeRange): PresetKey {
  const presets: Exclude<PresetKey, 'custom'>[] = ['today', 'thisWeek', 'thisMonth', 'last7days', 'last30days'];
  for (const preset of presets) {
    const presetRange = buildPresetRange(preset);
    if (sameMinute(range.from, presetRange.from) && sameMinute(range.to, presetRange.to)) {
      return preset;
    }
  }

  return 'custom';
}

export function defaultRangeToday(): DateTimeRange {
  return buildPresetRange('today');
}

export function DateTimeRangePicker({ value, onChange, className }: DateTimeRangePickerProps) {
  const [open, setOpen] = useState(false);
  const [fromInput, setFromInput] = useState(toLocalInputValue(value.from));
  const [toInput, setToInput] = useState(toLocalInputValue(value.to));

  useEffect(() => {
    setFromInput(toLocalInputValue(value.from));
    setToInput(toLocalInputValue(value.to));
  }, [value.from.getTime(), value.to.getTime()]);

  const selectedPreset = useMemo(
    () => detectPreset(value),
    [value.from.getTime(), value.to.getTime()]
  );

  const triggerLabel = useMemo(() => {
    if (selectedPreset !== 'custom') {
      return PRESET_LABELS[selectedPreset];
    }

    return `${format(value.from, 'yyyy-MM-dd HH:mm')} 至 ${format(value.to, 'yyyy-MM-dd HH:mm')}`;
  }, [selectedPreset, value.from, value.to]);

  const updateFrom = (next: Date) => {
    if (next >= value.to) {
      onChange({ from: next, to: new Date(next.getTime() + 60_000) });
      return;
    }
    onChange({ from: next, to: value.to });
  };

  const updateTo = (next: Date) => {
    if (next <= value.from) {
      onChange({ from: new Date(next.getTime() - 60_000), to: next });
      return;
    }
    onChange({ from: value.from, to: next });
  };

  const handleFromInputChange = (nextValue: string) => {
    setFromInput(nextValue);
    const parsed = parseLocalInputValue(nextValue);
    if (parsed) {
      updateFrom(parsed);
    }
  };

  const handleToInputChange = (nextValue: string) => {
    setToInput(nextValue);
    const parsed = parseLocalInputValue(nextValue);
    if (parsed) {
      updateTo(parsed);
    }
  };

  const handlePresetChange = (preset: PresetKey) => {
    if (preset === 'custom') {
      return;
    }
    onChange(buildPresetRange(preset));
    setOpen(false);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            'h-11 w-full justify-between gap-3 rounded-xl border-border/80 bg-background/90 px-3 text-sm shadow-sm transition-colors hover:bg-background md:min-w-[280px]',
            className
          )}
        >
          <span className="flex min-w-0 items-center gap-2">
            <CalendarRange className="h-4 w-4 shrink-0 opacity-65" />
            <span className="truncate text-left">{triggerLabel}</span>
          </span>
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[min(720px,calc(100vw-2rem))] rounded-2xl border-border/80 bg-popover/95 p-4 backdrop-blur-xl">
        <div className="space-y-4">
          <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
            <label className="sr-only" htmlFor="range-from-input">
              开始时间
            </label>
            <input
              id="range-from-input"
              type="text"
              inputMode="text"
              autoComplete="off"
              placeholder="yyyy-MM-dd HH:mm"
              pattern="\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}"
              value={fromInput}
              onChange={(event) => handleFromInputChange(event.target.value)}
              className="h-11 min-w-0 flex-1 rounded-xl border border-border/80 bg-background/90 px-4 text-sm shadow-sm outline-none ring-offset-background transition-colors focus-visible:ring-2 focus-visible:ring-ring sm:px-3 md:text-sm"
            />
            <span className="shrink-0 text-center text-xs font-medium text-muted-foreground">至</span>
            <label className="sr-only" htmlFor="range-to-input">
              结束时间
            </label>
            <input
              id="range-to-input"
              type="text"
              inputMode="text"
              autoComplete="off"
              placeholder="yyyy-MM-dd HH:mm"
              pattern="\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}"
              value={toInput}
              onChange={(event) => handleToInputChange(event.target.value)}
              className="h-11 min-w-0 flex-1 rounded-xl border border-border/80 bg-background/90 px-4 text-sm shadow-sm outline-none ring-offset-background transition-colors focus-visible:ring-2 focus-visible:ring-ring sm:px-3 md:text-sm"
            />
          </div>

          <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
            {(
              [
                ['today', '今天'],
                ['thisWeek', '本周'],
                ['thisMonth', '本月'],
                ['last7days', '过去7天'],
                ['last30days', '过去30天'],
                ['custom', '自定义'],
              ] as [PresetKey, string][]
            ).map(([preset, label]) => (
              <Button
                key={preset}
                type="button"
                variant={selectedPreset === preset ? 'default' : 'outline'}
                onClick={() => handlePresetChange(preset)}
                className="h-10 rounded-xl"
              >
                {label}
              </Button>
            ))}
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );
}
