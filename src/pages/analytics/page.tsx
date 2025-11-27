'use client';

import { Card } from '@/components/ui/card';

export default function AnalyticsPage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Analytics</h1>
        <p className="text-muted-foreground mt-1">Track performance and insights</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card className="bg-card border-border p-6">
          <h2 className="text-lg font-semibold text-foreground mb-4">Performance</h2>
          <div className="h-64 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Performance chart placeholder</p>
          </div>
        </Card>

        <Card className="bg-card border-border p-6">
          <h2 className="text-lg font-semibold text-foreground mb-4">Statistics</h2>
          <div className="space-y-4">
            <div className="flex justify-between">
              <p className="text-muted-foreground">Win Rate</p>
              <p className="text-foreground font-medium">68%</p>
            </div>
            <div className="flex justify-between">
              <p className="text-muted-foreground">Total Trades</p>
              <p className="text-foreground font-medium">234</p>
            </div>
            <div className="flex justify-between">
              <p className="text-muted-foreground">Avg Return</p>
              <p className="text-accent font-medium">+2.4%</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
