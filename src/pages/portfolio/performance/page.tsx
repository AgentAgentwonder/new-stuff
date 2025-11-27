'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function PerformancePage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Performance</h1>
        <p className="text-muted-foreground mt-1">Track your portfolio performance metrics</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card className="bg-card border-border">
          <CardContent className="pt-6">
            <p className="text-xs text-muted-foreground">Total Return</p>
            <p className="text-2xl font-bold text-accent mt-2">+24.5%</p>
          </CardContent>
        </Card>
        <Card className="bg-card border-border">
          <CardContent className="pt-6">
            <p className="text-xs text-muted-foreground">Win Rate</p>
            <p className="text-2xl font-bold text-accent mt-2">68%</p>
          </CardContent>
        </Card>
        <Card className="bg-card border-border">
          <CardContent className="pt-6">
            <p className="text-xs text-muted-foreground">Sharpe Ratio</p>
            <p className="text-2xl font-bold text-accent mt-2">1.82</p>
          </CardContent>
        </Card>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Performance Chart</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-80 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Performance chart will be integrated here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
