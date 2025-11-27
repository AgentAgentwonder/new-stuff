'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

export default function SentimentPage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Market Sentiment</h1>
        <p className="text-muted-foreground mt-1">
          Overall market sentiment and social media trends
        </p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Sentiment Analysis</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-96 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Sentiment analysis will appear here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
