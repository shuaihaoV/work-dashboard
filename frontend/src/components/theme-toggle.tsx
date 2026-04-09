import { Moon, Sun, SunMoon } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { useTheme } from '@/theme';

export function ThemeToggle() {
  const { mode, cycleMode } = useTheme();

  const Icon = mode === 'system' ? SunMoon : mode === 'light' ? Sun : Moon;

  return (
    <Button
      type="button"
      variant="outline"
      size="sm"
      onClick={cycleMode}
      className="h-11 w-11 rounded-xl border-border/80 bg-background/90 p-0 shadow-sm transition-colors hover:bg-background"
      title="点击切换主题：跟随系统 → 浅色 → 深色"
    >
      <Icon className="h-4 w-4" />
      <span className="sr-only">切换主题</span>
    </Button>
  );
}
