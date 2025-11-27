'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { TrendingUp, Flame, Eye, BarChart3 } from 'lucide-react';

export default function MarketSurveillancePage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Market Surveillance</h1>
        <p className="text-muted-foreground mt-1">
          Monitor market trends, fresh coins, and trading activity
        </p>
      </div>

      <Tabs defaultValue="trends" className="w-full">
        <TabsList className="grid grid-cols-4 w-full">
          <TabsTrigger value="trends" className="flex items-center gap-2">
            <TrendingUp className="w-4 h-4" />
            <span className="hidden sm:inline">Trends</span>
          </TabsTrigger>
          <TabsTrigger value="fresh-coins" className="flex items-center gap-2">
            <Flame className="w-4 h-4" />
            <span className="hidden sm:inline">Fresh Coins</span>
          </TabsTrigger>
          <TabsTrigger value="buyers" className="flex items-center gap-2">
            <Eye className="w-4 h-4" />
            <span className="hidden sm:inline">Fresh Buyers</span>
          </TabsTrigger>
          <TabsTrigger value="watchlist" className="flex items-center gap-2">
            <BarChart3 className="w-4 h-4" />
            <span className="hidden sm:inline">Watchlist</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="trends" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Market Trends</CardTitle>
              <CardDescription>Current market sentiment and trending movements</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Trend data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="fresh-coins" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Fresh Coins</CardTitle>
              <CardDescription>Newly listed coins and recent launches</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Fresh coins data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="buyers" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Fresh Buyers & Whale Tracking</CardTitle>
              <CardDescription>Monitor whale activity and large transactions</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Fresh buyers data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="watchlist" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Market Watchlist</CardTitle>
              <CardDescription>Your custom market watchlists and alerts</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Watchlist data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
