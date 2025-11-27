import { useCallback, useEffect } from 'react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { useAiStore } from '@/store/aiStore';
import { useShallow } from '@/store/createBoundStore';
import { AlertTriangle, X } from 'lucide-react';
import { Button } from '@/components/ui/button';

export function RiskBanner() {
  const aiSelector = useCallback(
    (state: ReturnType<typeof useAiStore.getState>) => ({
      patternWarnings: state.patternWarnings,
      fetchPatternWarnings: state.fetchPatternWarnings,
      dismissPatternWarning: state.dismissPatternWarning,
    }),
    []
  );
  const { patternWarnings, fetchPatternWarnings, dismissPatternWarning } = useAiStore(
    aiSelector,
    useShallow
  );

  useEffect(() => {
    fetchPatternWarnings();
  }, [fetchPatternWarnings]);

  const handleDismiss = useCallback(
    async (warningId: string) => {
      try {
        await dismissPatternWarning(warningId);
      } catch (err) {
        console.error('Failed to dismiss warning:', err);
      }
    },
    [dismissPatternWarning]
  );

  if (!patternWarnings || patternWarnings.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      {patternWarnings.map(warning => (
        <Alert
          key={warning.id}
          variant="destructive"
          className="bg-yellow-500/10 border-yellow-500/50 text-yellow-600 dark:text-yellow-400"
        >
          <AlertTriangle className="h-4 w-4" />
          <AlertTitle className="flex items-center justify-between">
            <span>Risk Warning: {warning.pattern}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => handleDismiss(warning.id)}
              className="h-6 w-6 p-0 hover:bg-yellow-500/20"
            >
              <X className="h-3 w-3" />
            </Button>
          </AlertTitle>
          <AlertDescription className="mt-2">
            {warning.description}
            <br />
            <span className="text-xs mt-1 inline-block">
              Confidence: {(warning.confidence * 100).toFixed(0)}%
            </span>
          </AlertDescription>
        </Alert>
      ))}
    </div>
  );
}
