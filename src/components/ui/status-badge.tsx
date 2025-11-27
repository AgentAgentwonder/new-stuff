import { Badge } from './badge';
import { cn } from '@/lib/utils';

export type StatusType =
  | 'pending'
  | 'filled'
  | 'cancelled'
  | 'failed'
  | 'active'
  | 'success'
  | 'error'
  | 'warning'
  | 'info';

interface StatusBadgeProps {
  status: StatusType;
  label?: string;
  className?: string;
}

const statusConfig: Record<
  StatusType,
  { variant: 'default' | 'secondary' | 'destructive' | 'outline'; className: string }
> = {
  pending: { variant: 'outline', className: 'bg-blue-500/10 text-blue-500 border-blue-500/50' },
  filled: { variant: 'default', className: 'bg-accent/10 text-accent border-accent/50' },
  cancelled: {
    variant: 'secondary',
    className: 'bg-muted-foreground/10 text-muted-foreground border-muted-foreground/50',
  },
  failed: {
    variant: 'destructive',
    className: 'bg-destructive/10 text-destructive border-destructive/50',
  },
  active: { variant: 'default', className: 'bg-accent/10 text-accent border-accent/50' },
  success: { variant: 'default', className: 'bg-accent/10 text-accent border-accent/50' },
  error: {
    variant: 'destructive',
    className: 'bg-destructive/10 text-destructive border-destructive/50',
  },
  warning: {
    variant: 'outline',
    className: 'bg-yellow-500/10 text-yellow-500 border-yellow-500/50',
  },
  info: { variant: 'outline', className: 'bg-blue-500/10 text-blue-500 border-blue-500/50' },
};

export function StatusBadge({ status, label, className }: StatusBadgeProps) {
  const config = statusConfig[status];
  const displayLabel = label || status.charAt(0).toUpperCase() + status.slice(1);

  return (
    <Badge variant={config.variant} className={cn('border', config.className, className)}>
      {displayLabel}
    </Badge>
  );
}
