import { useDeferredValue, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Check } from 'lucide-react';

import { type ApiResponse } from '@/api';
import { Button } from '@/components/ui/button';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from '@/components/ui/command';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { cn } from '@/lib/utils';

interface FilterComboboxProps<T> {
  value: T | null;
  onChange: (value: T | null) => void;
  fetchOptions: (keyword: string) => Promise<ApiResponse<T[]>>;
  queryKeyPrefix: string;
  getOptionKey: (option: T) => string | number;
  getOptionLabel: (option: T) => string;
  allLabel: string;
  searchPlaceholder: string;
  className?: string;
}

export function FilterCombobox<T>({
  value,
  onChange,
  fetchOptions,
  queryKeyPrefix,
  getOptionKey,
  getOptionLabel,
  allLabel,
  searchPlaceholder,
  className,
}: FilterComboboxProps<T>) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const deferredSearch = useDeferredValue(search);

  const optionsQuery = useQuery({
    queryKey: [queryKeyPrefix, deferredSearch.trim()],
    queryFn: () => fetchOptions(deferredSearch),
    enabled: open,
    staleTime: 60_000,
  });

  const options = optionsQuery.data?.data ?? [];
  const selectedKey = value ? String(getOptionKey(value)) : null;

  const handleSelect = (next: T | null) => {
    onChange(next);
    setSearch('');
    setOpen(false);
  };

  return (
    <Popover
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) {
          setSearch('');
        }
      }}
    >
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            'h-11 w-full justify-between gap-2 rounded-xl border-border/80 bg-background/90 px-3 text-sm shadow-sm transition-colors hover:bg-background md:w-full',
            className
          )}
        >
          <span className="truncate text-left">{value ? getOptionLabel(value) : allLabel}</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-[320px] overflow-hidden bg-popover p-0">
        <Command shouldFilter={false}>
          <CommandInput value={search} onValueChange={setSearch} placeholder={searchPlaceholder} />
          <CommandList>
            <CommandEmpty>{optionsQuery.isFetching ? '正在加载...' : '未找到匹配项'}</CommandEmpty>
            <CommandGroup heading="筛选范围">
              <CommandItem value="__all__" onSelect={() => handleSelect(null)}>
                <Check className={cn('h-4 w-4', value === null ? 'opacity-100' : 'opacity-0')} />
                <span>{allLabel}</span>
              </CommandItem>
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup heading={deferredSearch.trim() ? '匹配结果' : '常用选项'}>
              {options.map((option) => {
                const optionKey = String(getOptionKey(option));
                return (
                  <CommandItem key={optionKey} value={`${optionKey}:${getOptionLabel(option)}`} onSelect={() => handleSelect(option)}>
                    <Check className={cn('h-4 w-4', selectedKey === optionKey ? 'opacity-100' : 'opacity-0')} />
                    <span className="truncate font-medium">{getOptionLabel(option)}</span>
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
