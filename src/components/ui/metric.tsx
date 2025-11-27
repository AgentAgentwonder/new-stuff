import { Card, CardContent } from './card';
import { Skeleton } from './skeleton';
import { cn } from '@/lib/utils';

interface MetricProps {
  label: string;
  value: string | number;
  change?: string;
  changeType?: 'positive' | 'negative' | 'neutral';
  icon?: React.ReactNode;
  isLoading?: boolean;
  className?: string;
}

export function Metric({
  label,
  value,
  change,
  changeType = 'neutral',
  icon,
  isLoading,
  className,
}: MetricProps) {
  if (isLoading) {
    return (
      <Card className={cn('bg-card border-border', className)}>
        <CardContent className="p-4 space-y-2">
          <Skeleton className="h-4 w-24" />
          <Skeleton className="h-8 w-32" />
          {change && <Skeleton className="h-3 w-16" />}
        </CardContent>
      </Card>
    );
  }

  const changeColor =
    changeType === 'positive'
      ? 'text-accent'
      : changeType === 'negative'
        ? 'text-destructive'
        : 'text-muted-foreground';

  return (
    <Card className={cn('bg-card border-border', className)}>
      <CardContent className="p-4 space-y-2">
        <div className="flex items-center gap-2">
          {icon && <div className="text-muted-foreground">{icon}</div>}
          <p className="text-sm text-muted-foreground">{label}</p>
        </div>
        <p className="text-2xl font-bold text-foreground">{value}</p>
        {change && <p className={cn('text-sm font-medium', changeColor)}>{change}</p>}
      </CardContent>
    </Card>
  );
}
