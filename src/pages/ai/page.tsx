'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Brain, AlertTriangle, MessageCircle } from 'lucide-react';

export default function AIAnalysisPage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-foreground">AI Analysis</h1>
        <p className="text-muted-foreground mt-1">
          AI-powered insights, predictions, and risk assessment
        </p>
      </div>

      <Tabs defaultValue="predictions" className="w-full">
        <TabsList className="grid grid-cols-3 w-full">
          <TabsTrigger value="predictions" className="flex items-center gap-2">
            <Brain className="w-4 h-4" />
            <span className="hidden sm:inline">Predictions</span>
          </TabsTrigger>
          <TabsTrigger value="risk" className="flex items-center gap-2">
            <AlertTriangle className="w-4 h-4" />
            <span className="hidden sm:inline">Risk Scores</span>
          </TabsTrigger>
          <TabsTrigger value="assistant" className="flex items-center gap-2">
            <MessageCircle className="w-4 h-4" />
            <span className="hidden sm:inline">AI Chat</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="predictions" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Market Predictions</CardTitle>
              <CardDescription>AI-generated price predictions and market analysis</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Prediction data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="risk" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Risk Assessment</CardTitle>
              <CardDescription>Risk scores for assets and trading pairs</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">Risk data will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="assistant" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>AI Assistant</CardTitle>
              <CardDescription>Chat with our AI for trading advice and insights</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-muted-foreground">AI chat interface will appear here</div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
