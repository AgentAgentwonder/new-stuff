import { useEffect, useMemo } from 'react';
import { Metric } from '@/components/ui/metric';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { SkeletonTable } from '@/components/ui/skeleton-table';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { StatusBadge } from '@/components/ui/status-badge';
import { useWalletStore } from '@/store/walletStore';
import { useTradingStore } from '@/store/tradingStore';
import { usePortfolioStore } from '@/store/portfolioStore';
import { useAiStore } from '@/store/aiStore';
import { usePaperTrading } from '@/store';
import { Wallet, TrendingUp, AlertCircle, Activity } from 'lucide-react';

export default function Dashboard() {
  // Wallet selectors - primitive returns
  const activeAccount = useWalletStore(state => state.activeAccount);
  const balances = useWalletStore(state => state.balances);
  const fetchBalances = useWalletStore(state => state.fetchBalances);
  const walletLoading = useWalletStore(state => state.isLoading);
  const walletError = useWalletStore(state => state.error);

  // Trading selectors - primitive returns
  const activeOrders = useTradingStore(state => state.activeOrders);
  const getActiveOrders = useTradingStore(state => state.getActiveOrders);
  const tradingLoading = useTradingStore(state => state.isLoading);
  const tradingError = useTradingStore(state => state.error);

  // Portfolio selectors - primitive returns
  const totalValue = usePortfolioStore(state => state.totalValue);
  const totalPnl = usePortfolioStore(state => state.totalPnl);
  const totalPnlPercent = usePortfolioStore(state => state.totalPnlPercent);
  const fetchAnalytics = usePortfolioStore(state => state.fetchAnalytics);
  const portfolioLoading = usePortfolioStore(state => state.isLoading);
  const portfolioError = usePortfolioStore(state => state.error);

  // AI selectors - primitive returns
  const patternWarnings = useAiStore(state => state.patternWarnings);
  const fetchPatternWarnings = useAiStore(state => state.fetchPatternWarnings);
  const aiLoading = useAiStore(state => state.isLoading);
  const aiError = useAiStore(state => state.error);

  // Settings selectors - paper trading
  const { enabled: paperTradingEnabled, balance: paperTradingBalance } = usePaperTrading();

  useEffect(() => {
    if (activeAccount) {
      fetchBalances(activeAccount.publicKey);
      getActiveOrders(activeAccount.publicKey);
      fetchAnalytics(activeAccount.publicKey);
    }
    fetchPatternWarnings();
  }, [activeAccount, fetchBalances, getActiveOrders, fetchAnalytics, fetchPatternWarnings]);

  const accountBalances = useMemo(() => {
    if (!activeAccount) return [];
    return balances[activeAccount.publicKey] || [];
  }, [activeAccount, balances]);

  const totalBalance = useMemo(() => {
    return accountBalances.reduce((sum, token) => sum + token.uiAmount * (token.price || 0), 0);
  }, [accountBalances]);

  const pnlChangeType = useMemo(() => {
    if (totalPnl > 0) return 'positive';
    if (totalPnl < 0) return 'negative';
    return 'neutral';
  }, [totalPnl]);

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Dashboard</h1>
        <p className="text-muted-foreground mt-1">Welcome back to Eclipse Market Pro</p>
      </div>

      {/* Paper Trading Banner */}
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

      {/* Metrics Row */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Metric
          label="Total Balance"
          value={`$${totalBalance.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
          icon={<Wallet className="h-4 w-4" />}
          isLoading={walletLoading}
        />
        <Metric
          label="Portfolio Value"
          value={`$${totalValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
          icon={<TrendingUp className="h-4 w-4" />}
          isLoading={portfolioLoading}
        />
        <Metric
          label="Total P&L"
          value={`$${totalPnl.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`}
          change={`${totalPnlPercent >= 0 ? '+' : ''}${totalPnlPercent.toFixed(2)}%`}
          changeType={pnlChangeType}
          icon={<Activity className="h-4 w-4" />}
          isLoading={portfolioLoading}
        />
        <Metric
          label="Active Orders"
          value={activeOrders?.length || 0}
          icon={<Activity className="h-4 w-4" />}
          isLoading={tradingLoading}
        />
      </div>

      {/* Errors */}
      {(walletError || tradingError || portfolioError || aiError) && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            {walletError || tradingError || portfolioError || aiError}
          </AlertDescription>
        </Alert>
      )}

      {/* Pattern Warnings */}
      {patternWarnings && patternWarnings.length > 0 && (
        <Card className="bg-card border-yellow-500/50">
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2 text-yellow-600 dark:text-yellow-400">
              <AlertCircle className="h-5 w-5" />
              AI Pattern Warnings
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {patternWarnings.slice(0, 3).map(warning => (
                <div
                  key={warning.id}
                  className="p-3 bg-yellow-500/10 border border-yellow-500/30 rounded text-sm"
                >
                  <p className="font-medium text-foreground">{warning.pattern}</p>
                  <p className="text-xs text-muted-foreground mt-1">{warning.description}</p>
                  <p className="text-xs text-yellow-600 dark:text-yellow-400 mt-1">
                    Confidence: {(warning.confidence * 100).toFixed(0)}%
                  </p>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Token Balances */}
        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle className="text-lg">Token Balances</CardTitle>
          </CardHeader>
          <CardContent>
            {walletLoading && accountBalances.length === 0 ? (
              <SkeletonTable rows={5} columns={3} showHeader={false} />
            ) : accountBalances.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <p>No tokens found</p>
                {!activeAccount && (
                  <p className="text-xs mt-2">Connect your wallet to see balances</p>
                )}
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="text-left py-2 text-muted-foreground font-medium">Token</th>
                      <th className="text-right py-2 text-muted-foreground font-medium">Balance</th>
                      <th className="text-right py-2 text-muted-foreground font-medium">Value</th>
                    </tr>
                  </thead>
                  <tbody>
                    {accountBalances.slice(0, 5).map((token, idx) => (
                      <tr
                        key={idx}
                        className="border-b border-border hover:bg-muted/5 transition-colors"
                      >
                        <td className="py-3 text-foreground font-medium">{token.symbol}</td>
                        <td className="text-right text-foreground font-mono">
                          {token.uiAmount.toFixed(6)}
                        </td>
                        <td className="text-right text-foreground">
                          $
                          {(token.uiAmount * (token.price || 0)).toLocaleString('en-US', {
                            minimumFractionDigits: 2,
                            maximumFractionDigits: 2,
                          })}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Recent Orders */}
        <Card className="bg-card border-border">
          <CardHeader>
            <CardTitle className="text-lg">Recent Orders</CardTitle>
          </CardHeader>
          <CardContent>
            {tradingLoading && activeOrders.length === 0 ? (
              <SkeletonTable rows={5} columns={3} showHeader={false} />
            ) : activeOrders.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <p>No active orders</p>
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-border">
                      <th className="text-left py-2 text-muted-foreground font-medium">Status</th>
                      <th className="text-left py-2 text-muted-foreground font-medium">Pair</th>
                      <th className="text-right py-2 text-muted-foreground font-medium">Amount</th>
                    </tr>
                  </thead>
                  <tbody>
                    {activeOrders.slice(0, 5).map(order => (
                      <tr
                        key={order.id}
                        className="border-b border-border hover:bg-muted/5 transition-colors"
                      >
                        <td className="py-3">
                          <StatusBadge status={order.status} />
                        </td>
                        <td className="py-3 text-foreground">
                          {order.inputSymbol}/{order.outputSymbol}
                        </td>
                        <td className="text-right text-foreground font-mono">
                          {order.amount.toFixed(6)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
