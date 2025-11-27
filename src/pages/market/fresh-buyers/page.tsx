'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function FreshBuyersPage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Fresh Buyers & Whales</h1>
        <p className="text-muted-foreground mt-1">Monitor whale activity and large transactions</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Whale Activity</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Whale tracking data will appear here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
