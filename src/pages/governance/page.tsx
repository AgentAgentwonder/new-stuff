'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Vote, Mic, Bell } from 'lucide-react';

export default function GovernancePage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Governance & Alerts</h1>
        <p className="text-muted-foreground mt-1">
          Vote on proposals, manage alerts, and voice controls
        </p>
      </div>

      <Tabs defaultValue="alerts" className="w-full">
        <TabsList className="grid grid-cols-3 w-full">
          <TabsTrigger value="alerts" className="flex items-center gap-2">
            <Bell className="w-4 h-4" />
            <span className="hidden sm:inline">Alerts</span>
          </TabsTrigger>
          <TabsTrigger value="proposals" className="flex items-center gap-2">
            <Vote className="w-4 h-4" />
            <span className="hidden sm:inline">Proposals</span>
          </TabsTrigger>
          <TabsTrigger value="voice" className="flex items-center gap-2">
            <Mic className="w-4 h-4" />
            <span className="hidden sm:inline">Voice</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="alerts" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Smart Alerts</CardTitle>
              <CardDescription>Manage your price alerts and notifications</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Your alerts will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="proposals" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Governance Proposals</CardTitle>
              <CardDescription>Vote on platform proposals and governance decisions</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Active proposals will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="voice" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Voice Trading Controls</CardTitle>
              <CardDescription>Configure voice commands and settings</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Voice control settings will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
