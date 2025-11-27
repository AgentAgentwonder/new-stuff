'use client';

import { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';

export default function RecentTrades() {
  const [isUpdating, setIsUpdating] = useState(false);

  useEffect(() => {
    const interval = setInterval(() => {
      setIsUpdating(true);
      setTimeout(() => setIsUpdating(false), 200);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const trades = [
    { id: 1, pair: 'BTC/USD', type: 'Buy', price: '$98,234', time: '2 min ago' },
    { id: 2, pair: 'ETH/USD', type: 'Sell', price: '$3,456', time: '15 min ago' },
    { id: 3, pair: 'SOL/USD', type: 'Buy', price: '$234.50', time: '1 hour ago' },
    { id: 4, pair: 'BONK/USD', type: 'Buy', price: '$0.000045', time: '2 hours ago' },
    { id: 5, pair: 'JUP/USD', type: 'Sell', price: '$0.85', time: '3 hours ago' },
  ];

  return (
    <Card className="bg-card border-border p-3 h-full flex flex-col">
      {/* Header with Live Indicator */}
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-base font-semibold text-foreground">Recent Trades</h2>
        <div className="flex items-center gap-1">
          <div
            className={`w-2 h-2 rounded-full ${isUpdating ? 'bg-accent animate-pulse' : 'bg-accent/50'}`}
          />
          <span className="text-xs text-muted-foreground">Live</span>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto space-y-1 max-h-52">
        {trades.map(trade => (
          <div
            key={trade.id}
            className="flex justify-between items-center p-1 border-b border-border last:border-0 hover:bg-muted/5 transition-colors rounded text-xs"
          >
            <div>
              <p className="text-sm font-medium text-foreground">{trade.pair}</p>
              <p className="text-xs text-muted-foreground">{trade.time}</p>
            </div>
            <div className="text-right">
              <p
                className={`text-sm font-medium ${trade.type === 'Buy' ? 'text-accent' : 'text-destructive'}`}
              >
                {trade.type}
              </p>
              <p className="text-xs text-muted-foreground">{trade.price}</p>
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}
