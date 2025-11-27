'use client';

import { Card } from '@/components/ui/card';

export default function PortfolioOverview() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-4 gap-2">
      <Card className="bg-card border-border p-3">
        <div className="space-y-1">
          <p className="text-muted-foreground text-xs">Total Balance</p>
          <p className="text-xl font-bold text-foreground">$124,583.45</p>
          <p className="text-accent text-xs">+2.5% today</p>
        </div>
      </Card>

      <Card className="bg-card border-border p-3">
        <div className="space-y-1">
          <p className="text-muted-foreground text-xs">Available</p>
          <p className="text-xl font-bold text-foreground">$45,230.12</p>
          <p className="text-muted-foreground text-xs">Ready to trade</p>
        </div>
      </Card>

      <Card className="bg-card border-border p-3">
        <div className="space-y-1">
          <p className="text-muted-foreground text-xs">24h Change</p>
          <p className="text-xl font-bold text-accent">+$3,245.50</p>
          <p className="text-muted-foreground text-xs">+2.67%</p>
        </div>
      </Card>

      <Card className="bg-card border-border p-3">
        <div className="space-y-1">
          <p className="text-muted-foreground text-xs">Open Orders</p>
          <p className="text-xl font-bold text-foreground">12</p>
          <p className="text-muted-foreground text-xs">Active positions</p>
        </div>
      </Card>
    </div>
  );
}
