'use client';

import { Card, CardContent } from '@/components/ui/card';
import { Plus, Users } from 'lucide-react';

export default function WorkspacesPage() {
  const workspaces = [
    { id: 1, name: 'Personal Trading', members: 1, role: 'Owner' },
    { id: 2, name: 'Team Operations', members: 5, role: 'Admin' },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">Workspaces</h1>
          <p className="text-muted-foreground mt-1">Manage your workspaces and collaboration</p>
        </div>
        <button className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity">
          <Plus className="w-4 h-4" />
          New Workspace
        </button>
      </div>

      <div className="grid gap-4">
        {workspaces.map(workspace => (
          <Card key={workspace.id} className="bg-card border-border">
            <CardContent className="pt-6">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-semibold text-foreground">{workspace.name}</p>
                  <p className="text-sm text-muted-foreground mt-1 flex items-center gap-1">
                    <Users className="w-4 h-4" />
                    {workspace.members} member{workspace.members > 1 ? 's' : ''} • {workspace.role}
                  </p>
                </div>
                <button className="px-3 py-1.5 border border-border text-foreground rounded hover:bg-muted/20 transition-colors text-sm">
                  View
                </button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
