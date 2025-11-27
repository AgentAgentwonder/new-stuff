import type { ReactNode } from 'react';
import { useEffect, useState } from 'react';
import { errorLogger, type ErrorLog } from '@/utils/errorLogger';

interface DevErrorDisplayProps {
  children: ReactNode;
}

/**
 * Development error display component
 * Shows errors that occur during initialization and development
 * Displays at the root level to catch all errors
 */
export function DevErrorDisplay({ children }: DevErrorDisplayProps) {
  const [errors, setErrors] = useState<ErrorLog[]>([]);
  const [showDetails, setShowDetails] = useState(false);
  const [isExpanded, setIsExpanded] = useState(true);
  const [lastLogCount, setLastLogCount] = useState(0);

  // Subscribe to error logger updates - only update when logs actually change
  useEffect(() => {
    const interval = setInterval(() => {
      const logs = errorLogger.getLogs();
      // Only update state if the number of logs has changed
      if (logs.length !== lastLogCount) {
        setErrors(logs);
        setLastLogCount(logs.length);
      }
    }, 500); // Increased interval from 100ms to 500ms to reduce polling frequency

    return () => clearInterval(interval);
  }, [lastLogCount]);

  // Only show in development mode
  if (process.env.NODE_ENV !== 'development') {
    return <>{children}</>;
  }

  const hasErrors = errors.filter(log => log.type === 'error').length > 0;
  const hasWarnings = errors.filter(log => log.type === 'warning').length > 0;

  return (
    <>
      {children}

      {/* Error Summary Badge (always visible if errors exist) */}
      {(hasErrors || hasWarnings) && (
        <div className="fixed bottom-4 right-4 z-[9999]">
          <button
            onClick={() => {
              setIsExpanded(!isExpanded);
              if (!isExpanded) setShowDetails(false);
            }}
            className={`px-4 py-2 rounded-lg text-sm font-semibold text-white transition-all ${
              hasErrors ? 'bg-error hover:bg-red-600' : 'bg-warning hover:bg-yellow-600'
            } shadow-lg`}
          >
            {hasErrors &&
              `❌ ${errors.filter(log => log.type === 'error').length} Error${errors.filter(log => log.type === 'error').length !== 1 ? 's' : ''}`}
            {!hasErrors &&
              hasWarnings &&
              `⚠️ ${errors.filter(log => log.type === 'warning').length} Warning${errors.filter(log => log.type === 'warning').length !== 1 ? 's' : ''}`}
          </button>

          {/* Error Details Panel */}
          {isExpanded && (
            <div className="absolute bottom-14 right-0 mt-2 w-96 bg-background-secondary border border-error/30 rounded-lg shadow-xl overflow-hidden">
              {/* Header */}
              <div className="bg-error/10 border-b border-error/30 px-4 py-3 flex items-center justify-between">
                <h3 className="font-semibold text-error">Error Logs</h3>
                <button
                  onClick={() => {
                    setShowDetails(!showDetails);
                  }}
                  className="text-xs text-text-secondary hover:text-text px-2 py-1 rounded hover:bg-background transition-colors"
                >
                  {showDetails ? 'Hide' : 'Show'} Details
                </button>
              </div>

              {/* Logs List */}
              <div className="max-h-80 overflow-y-auto">
                {errors.length === 0 ? (
                  <div className="px-4 py-3 text-sm text-text-secondary">No errors logged</div>
                ) : (
                  errors.map((log, index) => (
                    <div
                      key={index}
                      className={`px-4 py-3 border-b border-border/50 text-xs ${
                        log.type === 'error'
                          ? 'bg-error/5'
                          : log.type === 'warning'
                            ? 'bg-warning/5'
                            : 'bg-success/5'
                      }`}
                    >
                      <div className="flex items-start gap-2">
                        <span
                          className={`flex-shrink-0 font-bold ${
                            log.type === 'error'
                              ? 'text-error'
                              : log.type === 'warning'
                                ? 'text-warning'
                                : 'text-success'
                          }`}
                        >
                          {log.type === 'error' ? '❌' : log.type === 'warning' ? '⚠️' : 'ℹ️'}
                        </span>
                        <div className="flex-1 min-w-0">
                          <div className="font-semibold text-text">{log.source}</div>
                          <div className="text-text-secondary break-words">{log.message}</div>
                          <div className="text-text-muted mt-1">
                            {new Date(log.timestamp).toLocaleTimeString()}
                          </div>

                          {showDetails && log.stack && (
                            <pre className="mt-2 bg-background p-2 rounded text-xs text-text-muted overflow-auto max-h-24 border border-border/50 font-mono">
                              {log.stack}
                            </pre>
                          )}

                          {showDetails && log.context && (
                            <pre className="mt-2 bg-background p-2 rounded text-xs text-text-muted overflow-auto max-h-24 border border-border/50 font-mono">
                              {JSON.stringify(log.context, null, 2)}
                            </pre>
                          )}
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>

              {/* Footer */}
              <div className="bg-background px-4 py-2 border-t border-border/50 flex gap-2 justify-end">
                <button
                  onClick={() => {
                    errorLogger.clear();
                    setErrors([]);
                  }}
                  className="text-xs px-3 py-1 rounded bg-background-tertiary hover:bg-background-hover text-text transition-colors"
                >
                  Clear
                </button>
                <button
                  onClick={() => {
                    const report = errorLogger.getErrorReport();
                    navigator.clipboard.writeText(report);
                  }}
                  className="text-xs px-3 py-1 rounded bg-background-tertiary hover:bg-background-hover text-text transition-colors"
                >
                  Copy Report
                </button>
              </div>
            </div>
          )}
        </div>
      )}
    </>
  );
}
