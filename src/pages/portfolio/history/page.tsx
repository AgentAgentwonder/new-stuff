'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function HistoryPage() {
  const trades = [
    {
      type: 'Buy',
      asset: 'BTC',
      amount: 0.5,
      price: '$43,200',
      date: '2024-01-15',
      status: 'Completed',
    },
    {
      type: 'Sell',
      asset: 'ETH',
      amount: 5,
      price: '$3,450',
      date: '2024-01-14',
      status: 'Completed',
    },
    {
      type: 'Buy',
      asset: 'SOL',
      amount: 100,
      price: '$234',
      date: '2024-01-13',
      status: 'Completed',
    },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Trading History</h1>
        <p className="text-muted-foreground mt-1">View your complete trading history</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Trade History</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left py-3 px-2 text-muted-foreground font-medium">Type</th>
                  <th className="text-left py-3 px-2 text-muted-foreground font-medium">Asset</th>
                  <th className="text-right py-3 px-2 text-muted-foreground font-medium">Amount</th>
                  <th className="text-right py-3 px-2 text-muted-foreground font-medium">Price</th>
                  <th className="text-left py-3 px-2 text-muted-foreground font-medium">Date</th>
                  <th className="text-left py-3 px-2 text-muted-foreground font-medium">Status</th>
                </tr>
              </thead>
              <tbody>
                {trades.map((trade, i) => (
                  <tr key={i} className="border-b border-border hover:bg-muted/5 transition-colors">
                    <td
                      className={`py-3 px-2 font-medium ${trade.type === 'Buy' ? 'text-accent' : 'text-destructive'}`}
                    >
                      {trade.type}
                    </td>
                    <td className="py-3 px-2 text-foreground">{trade.asset}</td>
                    <td className="text-right py-3 px-2 text-foreground">{trade.amount}</td>
                    <td className="text-right py-3 px-2 text-foreground">{trade.price}</td>
                    <td className="py-3 px-2 text-muted-foreground">{trade.date}</td>
                    <td className="py-3 px-2 text-accent">{trade.status}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
