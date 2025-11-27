'use client';

import { Card, CardContent } from '@/components/ui/card';
import { BookOpen, Play } from 'lucide-react';

export default function LearningPage() {
  const resources = [
    {
      id: 1,
      title: 'Getting Started with Spot Trading',
      type: 'Tutorial',
      duration: '5 min',
      icon: BookOpen,
    },
    {
      id: 2,
      title: 'Understanding Futures Trading',
      type: 'Video',
      duration: '12 min',
      icon: Play,
    },
    {
      id: 3,
      title: 'Risk Management Strategies',
      type: 'Guide',
      duration: '8 min',
      icon: BookOpen,
    },
  ];

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Learning Center</h1>
        <p className="text-muted-foreground mt-1">Tutorials, guides, and educational resources</p>
      </div>

      <div className="grid md:grid-cols-2 gap-4">
        {resources.map(resource => {
          const Icon = resource.icon;
          return (
            <Card
              key={resource.id}
              className="bg-card border-border cursor-pointer hover:border-primary transition-colors"
            >
              <CardContent className="pt-6">
                <div className="flex items-start justify-between">
                  <div>
                    <p className="font-semibold text-foreground">{resource.title}</p>
                    <p className="text-xs text-muted-foreground mt-2">{resource.type}</p>
                    <p className="text-xs text-muted-foreground mt-1">{resource.duration}</p>
                  </div>
                  <Icon className="w-5 h-5 text-accent flex-shrink-0" />
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
