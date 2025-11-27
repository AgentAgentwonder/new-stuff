'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function PredictionsPage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">AI Predictions</h1>
        <p className="text-muted-foreground mt-1">AI-powered market predictions and analysis</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Market Predictions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">AI predictions will appear here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
