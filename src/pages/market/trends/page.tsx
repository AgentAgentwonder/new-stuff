'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function TrendsPage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Market Trends</h1>
        <p className="text-muted-foreground mt-1">Track trending assets and market movements</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Top Trending</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Trending data will be integrated here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
