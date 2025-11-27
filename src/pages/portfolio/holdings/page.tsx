'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function HoldingsPage() {
  const holdings = [
    { symbol: 'BTC', amount: 2.5, value: '$247,500', change: '+5.2%', icon: '₿' },
    { symbol: 'ETH', amount: 15.2, value: '$52,500', change: '+3.1%', icon: 'Ξ' },
    { symbol: 'SOL', amount: 450, value: '$105,300', change: '+12.5%', icon: '◎' },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Holdings</h1>
        <p className="text-muted-foreground mt-1">View your cryptocurrency holdings</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Your Assets</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left py-3 px-2 text-muted-foreground font-medium">Asset</th>
                  <th className="text-right py-3 px-2 text-muted-foreground font-medium">Amount</th>
                  <th className="text-right py-3 px-2 text-muted-foreground font-medium">Value</th>
                  <th className="text-right py-3 px-2 text-muted-foreground font-medium">
                    24h Change
                  </th>
                </tr>
              </thead>
              <tbody>
                {holdings.map(holding => (
                  <tr
                    key={holding.symbol}
                    className="border-b border-border hover:bg-muted/5 transition-colors"
                  >
                    <td className="py-3 px-2 text-foreground font-medium">{holding.symbol}</td>
                    <td className="text-right py-3 px-2 text-foreground">{holding.amount}</td>
                    <td className="text-right py-3 px-2 text-foreground">{holding.value}</td>
                    <td
                      className={`text-right py-3 px-2 font-medium ${holding.change.includes('+') ? 'text-accent' : 'text-destructive'}`}
                    >
                      {holding.change}
                    </td>
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
