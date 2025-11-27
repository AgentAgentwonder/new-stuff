import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { usePaperTrading } from '@/store';
import { TrendingUp } from 'lucide-react';

export default function PaperTradingPage() {
  const { enabled, balance } = usePaperTrading();

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Paper Trading</h1>
        <p className="text-muted-foreground mt-1">Practice trading with virtual funds</p>
      </div>

      {!enabled && (
        <Card className="bg-card border-border border-accent">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <TrendingUp className="w-5 h-5 text-accent" />
              <div>
                <p className="text-sm font-medium text-foreground">
                  Paper Trading is currently disabled
                </p>
                <p className="text-xs text-muted-foreground mt-1">
                  Enable it in Settings to start practicing with virtual funds
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Practice Account</CardTitle>
          <CardDescription>
            {enabled
              ? 'Your virtual trading account'
              : 'Enable paper trading in settings to use this feature'}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-muted/10 rounded p-4">
                <p className="text-xs text-muted-foreground">Virtual Balance</p>
                <p className="text-2xl font-bold text-accent mt-1">
                  $
                  {balance.toLocaleString('en-US', {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 2,
                  })}
                </p>
              </div>
              <div className="bg-muted/10 rounded p-4">
                <p className="text-xs text-muted-foreground">P&L</p>
                <p className="text-2xl font-bold text-accent mt-1">$0.00</p>
              </div>
            </div>
            <div className="h-64 bg-muted/10 rounded flex items-center justify-center border border-border">
              <p className="text-muted-foreground">
                Paper trading interface will be integrated here
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
