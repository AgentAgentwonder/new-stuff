'use client';
import PortfolioOverview from '@/components/dashboard/portfolio-overview';
import RecentTrades from '@/components/dashboard/recent-trades';
import NewCoins from '@/components/dashboard/new-coins';

export default function DashboardPage() {
  return (
    <div className="p-4 space-y-3 h-screen overflow-hidden flex flex-col">
      {/* Header - Compact */}
      <div>
        <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
        <p className="text-xs text-muted-foreground">Welcome back to Eclipse Market Trading</p>
      </div>

      {/* Portfolio Stats - Compact */}
      <PortfolioOverview />

      {/* Main Content Grid - No Scroll */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-3 min-h-0">
        {/* New Coins - Takes 2 columns */}
        <div className="lg:col-span-2 min-h-0">
          <NewCoins />
        </div>

        {/* Recent Trades - Takes 1 column */}
        <div className="min-h-0">
          <RecentTrades />
        </div>
      </div>
    </div>
  );
}
