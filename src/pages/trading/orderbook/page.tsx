'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function OrderBookPage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Order Book</h1>
        <p className="text-muted-foreground mt-1">View live order book data and market depth</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Market Depth</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Order book interface will be integrated here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
