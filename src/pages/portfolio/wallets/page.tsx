'use client';

import { Card, CardContent } from '@/components/ui/card';
import { Copy, ExternalLink } from 'lucide-react';

export default function WalletsPage() {
  const wallets = [
    { type: 'Phantom', address: '7Kj9...q4M2', network: 'Solana', balance: '$4,850.00' },
    { type: 'MetaMask', address: '0x8a2...b3c', network: 'Ethereum', balance: '$12,450.00' },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Wallets</h1>
        <p className="text-muted-foreground mt-1">Manage your connected wallets</p>
      </div>

      <div className="space-y-4">
        {wallets.map((wallet, i) => (
          <Card key={i} className="bg-card border-border">
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-semibold text-foreground">{wallet.type}</p>
                  <p className="text-sm text-muted-foreground mt-1">{wallet.network}</p>
                  <p className="text-xs text-muted-foreground mt-1 font-mono">{wallet.address}</p>
                </div>
                <div className="text-right">
                  <p className="text-lg font-bold text-accent">{wallet.balance}</p>
                  <div className="flex gap-2 mt-2">
                    <button className="p-2 hover:bg-muted/20 rounded transition-colors">
                      <Copy className="w-4 h-4 text-muted-foreground" />
                    </button>
                    <button className="p-2 hover:bg-muted/20 rounded transition-colors">
                      <ExternalLink className="w-4 h-4 text-muted-foreground" />
                    </button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
