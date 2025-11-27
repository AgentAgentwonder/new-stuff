'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Mic } from 'lucide-react';

export default function VoicePage() {
  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Voice Trading Controls</h1>
        <p className="text-muted-foreground mt-1">Configure voice commands and settings</p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>Voice Control Settings</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-muted/10 rounded border border-border">
            <div className="flex items-center gap-3">
              <Mic className="w-5 h-5 text-accent" />
              <div>
                <p className="font-semibold text-foreground">Voice Recognition</p>
                <p className="text-sm text-muted-foreground">Enable voice commands for trading</p>
              </div>
            </div>
            <div className="w-12 h-6 bg-accent rounded-full" />
          </div>

          <div className="h-64 bg-muted/10 rounded flex items-center justify-center border border-border">
            <p className="text-muted-foreground">Voice control interface will be integrated here</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
