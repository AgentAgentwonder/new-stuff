'use client';

import { Card } from '@/components/ui/card';

export default function TradingPage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Trading</h1>
        <p className="text-muted-foreground mt-1">Execute trades and manage your positions</p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className="lg:col-span-3">
          <Card className="bg-card border-border p-6">
            <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
              <p className="text-muted-foreground">Trading interface placeholder</p>
            </div>
          </Card>
        </div>

        <div>
          <Card className="bg-card border-border p-6">
            <h2 className="text-lg font-semibold text-foreground mb-4">Order Form</h2>
            <div className="space-y-4">
              <div>
                <label className="text-sm text-muted-foreground">Pair</label>
                <div className="bg-input rounded px-3 py-2 text-foreground">BTC/USD</div>
              </div>
              <div>
                <label className="text-sm text-muted-foreground">Amount</label>
                <input
                  type="number"
                  placeholder="0.00"
                  className="w-full bg-input rounded px-3 py-2 text-foreground placeholder:text-muted-foreground"
                />
              </div>
              <div>
                <label className="text-sm text-muted-foreground">Price</label>
                <input
                  type="number"
                  placeholder="0.00"
                  className="w-full bg-input rounded px-3 py-2 text-foreground placeholder:text-muted-foreground"
                />
              </div>
              <button className="w-full bg-primary text-primary-foreground rounded py-2 font-medium hover:opacity-90 transition-opacity">
                Buy
              </button>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
