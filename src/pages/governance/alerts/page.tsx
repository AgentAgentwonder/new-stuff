'use client';

import { Card, CardContent } from '@/components/ui/card';
import { Bell, X } from 'lucide-react';

export default function AlertsPage() {
  const alerts = [
    { id: 1, type: 'Price Alert', asset: 'BTC', condition: 'Price > $50,000', status: 'Active' },
    { id: 2, type: 'Volume Alert', asset: 'ETH', condition: 'Volume > 1M USD', status: 'Active' },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Smart Alerts</h1>
        <p className="text-muted-foreground mt-1">Manage your alerts and notifications</p>
      </div>

      <div className="space-y-3">
        {alerts.map(alert => (
          <Card key={alert.id} className="bg-card border-border">
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <Bell className="w-5 h-5 text-accent" />
                  <div>
                    <p className="font-semibold text-foreground">{alert.type}</p>
                    <p className="text-sm text-muted-foreground">
                      {alert.asset}: {alert.condition}
                    </p>
                  </div>
                </div>
                <button className="p-2 hover:bg-destructive/20 rounded transition-colors">
                  <X className="w-4 h-4 text-destructive" />
                </button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
