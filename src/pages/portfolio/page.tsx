'use client';

import { Card } from '@/components/ui/card';

export default function PortfolioPage() {
  const holdings = [
    { symbol: 'BTC', amount: 2.5, value: '$247,500', change: '+5.2%' },
    { symbol: 'ETH', amount: 15.2, value: '$52,500', change: '+3.1%' },
    { symbol: 'SOL', amount: 450, value: '$105,300', change: '+12.5%' },
  ];

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Portfolio</h1>
        <p className="text-muted-foreground mt-1">View and manage your holdings</p>
      </div>

      <Card className="bg-card border-border p-6">
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-foreground">Holdings</h2>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left py-2 text-muted-foreground">Asset</th>
                  <th className="text-right py-2 text-muted-foreground">Amount</th>
                  <th className="text-right py-2 text-muted-foreground">Value</th>
                  <th className="text-right py-2 text-muted-foreground">Change</th>
                </tr>
              </thead>
              <tbody>
                {holdings.map(holding => (
                  <tr
                    key={holding.symbol}
                    className="border-b border-border hover:bg-muted/5 transition-colors"
                  >
                    <td className="py-3 text-foreground font-medium">{holding.symbol}</td>
                    <td className="text-right text-foreground">{holding.amount}</td>
                    <td className="text-right text-foreground">{holding.value}</td>
                    <td
                      className={`text-right font-medium ${holding.change.includes('+') ? 'text-accent' : 'text-destructive'}`}
                    >
                      {holding.change}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </Card>
    </div>
  );
}
