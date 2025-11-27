import { useCallback, useMemo } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { StatusBadge } from '@/components/ui/status-badge';
import { SkeletonTable } from '@/components/ui/skeleton-table';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useTradingStore } from '@/store/tradingStore';
import { useShallow } from '@/store/createBoundStore';
import { X, AlertCircle } from 'lucide-react';

export function OrderBlotter() {
  const tradingSelector = useCallback(
    (state: ReturnType<typeof useTradingStore.getState>) => ({
      activeOrders: state.activeOrders,
      optimisticOrders: state.optimisticOrders,
      cancelOrder: state.cancelOrder,
      isLoading: state.isLoading,
      error: state.error,
    }),
    []
  );
  const { activeOrders, optimisticOrders, cancelOrder, isLoading, error } = useTradingStore(
    tradingSelector,
    useShallow
  );

  const allOrders = useMemo(() => {
    const optimistic = Array.from(optimisticOrders.values());
    return [...optimistic, ...activeOrders];
  }, [optimisticOrders, activeOrders]);

  const handleCancel = useCallback(
    async (orderId: string) => {
      try {
        await cancelOrder(orderId);
      } catch (err) {
        console.error('Failed to cancel order:', err);
      }
    },
    [cancelOrder]
  );

  const formatAmount = (amount: number) => {
    return amount.toFixed(6);
  };

  const formatPrice = (price?: number) => {
    if (!price) return '-';
    return price.toFixed(6);
  };

  if (isLoading && allOrders.length === 0) {
    return <SkeletonTable rows={5} columns={6} />;
  }

  return (
    <Card className="bg-card border-border">
      <CardHeader>
        <CardTitle className="text-lg">Active Orders</CardTitle>
      </CardHeader>
      <CardContent>
        {error && (
          <Alert variant="destructive" className="mb-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {allOrders.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <p>No active orders</p>
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-border">
                    <th className="text-left py-2 px-2 text-muted-foreground font-medium">
                      Status
                    </th>
                    <th className="text-left py-2 px-2 text-muted-foreground font-medium">Side</th>
                    <th className="text-left py-2 px-2 text-muted-foreground font-medium">Type</th>
                    <th className="text-left py-2 px-2 text-muted-foreground font-medium">Pair</th>
                    <th className="text-right py-2 px-2 text-muted-foreground font-medium">
                      Amount
                    </th>
                    <th className="text-right py-2 px-2 text-muted-foreground font-medium">
                      Price
                    </th>
                    <th className="text-center py-2 px-2 text-muted-foreground font-medium">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {allOrders.map(order => (
                    <tr
                      key={order.id}
                      className="border-b border-border hover:bg-muted/5 transition-colors"
                    >
                      <td className="py-3 px-2">
                        <StatusBadge status={order.status} />
                      </td>
                      <td className="py-3 px-2">
                        <span
                          className={
                            order.side === 'buy'
                              ? 'text-accent font-medium'
                              : 'text-destructive font-medium'
                          }
                        >
                          {order.side.toUpperCase()}
                        </span>
                      </td>
                      <td className="py-3 px-2 text-foreground capitalize">{order.orderType}</td>
                      <td className="py-3 px-2 text-foreground">
                        {order.inputSymbol}/{order.outputSymbol}
                      </td>
                      <td className="py-3 px-2 text-right text-foreground font-mono">
                        {formatAmount(order.amount)}
                      </td>
                      <td className="py-3 px-2 text-right text-foreground font-mono">
                        {formatPrice(order.limitPrice)}
                      </td>
                      <td className="py-3 px-2 text-center">
                        {order.status === 'pending' && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleCancel(order.id)}
                            className="text-destructive hover:text-destructive hover:bg-destructive/10"
                          >
                            <X className="h-4 w-4" />
                          </Button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
