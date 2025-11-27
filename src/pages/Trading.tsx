import { useEffect } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { OrderForm } from '@/components/trading/OrderForm';
import { OrderBlotter } from '@/components/trading/OrderBlotter';
import { RiskBanner } from '@/components/trading/RiskBanner';
import { useTradingStore } from '@/store/tradingStore';
import { usePaperTrading } from '@/store';
import { AlertCircle, TrendingUp } from 'lucide-react';

export default function Trading() {
  // Trading selectors - primitive returns
  const isInitialized = useTradingStore(state => state.isInitialized);
  const initialize = useTradingStore(state => state.initialize);
  const error = useTradingStore(state => state.error);

  // Settings selectors - paper trading
  const { enabled: paperTradingEnabled, balance: paperTradingBalance } = usePaperTrading();

  useEffect(() => {
    if (!isInitialized) {
      initialize();
    }
  }, [isInitialized, initialize]);

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Trading</h1>
        <p className="text-muted-foreground mt-1">Execute trades and manage your positions</p>
      </div>

      {paperTradingEnabled && (
        <Alert className="bg-accent/10 border-accent">
          <TrendingUp className="h-4 w-4 text-accent" />
          <AlertDescription className="text-accent">
            Paper Trading Mode Active - Virtual Balance: $
            {paperTradingBalance.toLocaleString('en-US', {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}
          </AlertDescription>
        </Alert>
      )}

      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      <RiskBanner />

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className="lg:col-span-3 space-y-6">
          <OrderBlotter />
        </div>

        <div>
          <OrderForm />
        </div>
      </div>
    </div>
  );
}
