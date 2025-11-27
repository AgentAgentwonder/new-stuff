'use client';

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ThumbsUp, ThumbsDown } from 'lucide-react';

export default function ProposalsPage() {
  const proposals = [
    {
      id: 1,
      title: 'Increase Protocol Fee',
      description: 'Raise platform fee from 0.1% to 0.15%',
      votes: { yes: 7500, no: 2500 },
    },
    {
      id: 2,
      title: 'Add New Trading Pair',
      description: 'List XRP/USD trading pair',
      votes: { yes: 9200, no: 800 },
    },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Governance Proposals</h1>
        <p className="text-muted-foreground mt-1">Vote on platform decisions</p>
      </div>

      <div className="space-y-4">
        {proposals.map(proposal => (
          <Card key={proposal.id} className="bg-card border-border">
            <CardHeader>
              <CardTitle className="text-lg">{proposal.title}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">{proposal.description}</p>
              <div className="space-y-2">
                <div className="flex justify-between text-xs text-muted-foreground mb-1">
                  <span>Yes: {proposal.votes.yes.toLocaleString()}</span>
                  <span>No: {proposal.votes.no.toLocaleString()}</span>
                </div>
                <div className="flex h-2 gap-1 bg-muted rounded overflow-hidden">
                  <div
                    className="bg-accent"
                    style={{
                      width: `${(proposal.votes.yes / (proposal.votes.yes + proposal.votes.no)) * 100}%`,
                    }}
                  />
                  <div className="flex-1 bg-destructive/50" />
                </div>
              </div>
              <div className="flex gap-2 mt-4">
                <button className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity">
                  <ThumbsUp className="w-4 h-4" />
                  Vote Yes
                </button>
                <button className="flex-1 flex items-center justify-center gap-2 px-3 py-2 border border-destructive text-destructive rounded hover:bg-destructive/10 transition-colors">
                  <ThumbsDown className="w-4 h-4" />
                  Vote No
                </button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
