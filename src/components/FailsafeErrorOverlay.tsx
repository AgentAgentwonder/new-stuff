import { useEffect, useState } from 'react';
import { errorLogger, type ErrorLog } from '@/utils/errorLogger';

/**
 * Failsafe Error Overlay - Uses 100% inline styles to work even if CSS fails
 * Displays errors directly on screen with full details and restart capability
 * Persists across app restarts via localStorage
 */
export function FailsafeErrorOverlay() {
  const [errors, setErrors] = useState<ErrorLog[]>([]);
  const [isExpanded, setIsExpanded] = useState(false);
  const [showAllLogs, setShowAllLogs] = useState(false);
  const [lastLogCount, setLastLogCount] = useState(0);

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

  const errorCount = errors.filter(log => log.type === 'error').length;
  const warningCount = errors.filter(log => log.type === 'warning').length;

  if (errorCount === 0 && warningCount === 0) {
    return null;
  }

  const handleRestart = () => {
    window.location.reload();
  };

  const handleCopyAll = () => {
    const report = showAllLogs ? errorLogger.getFullErrorReport() : errorLogger.getErrorReport();
    navigator.clipboard
      .writeText(report)
      .then(() => {
        alert('Error report copied to clipboard!');
      })
      .catch(() => {
        alert('Failed to copy. Check console for error report.');
        console.log('ERROR REPORT:\n\n', report);
      });
  };

  const handleClear = () => {
    if (confirm('Clear all error logs? This cannot be undone.')) {
      errorLogger.clear();
      setErrors([]);
    }
  };

  const handleToggleLogs = () => {
    setShowAllLogs(!showAllLogs);
  };

  const displayLogs = showAllLogs ? errorLogger.getAllStoredLogs() : errors;

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '20px',
        right: '20px',
        zIndex: 999999,
        fontFamily: 'system-ui, -apple-system, sans-serif',
        fontSize: '14px',
      }}
    >
      {/* Floating Badge Button */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        style={{
          padding: '12px 20px',
          backgroundColor: errorCount > 0 ? '#ef4444' : '#f59e0b',
          color: '#ffffff',
          border: 'none',
          borderRadius: '8px',
          cursor: 'pointer',
          fontWeight: 'bold',
          fontSize: '14px',
          boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <span>{errorCount > 0 ? '❌' : '⚠️'}</span>
        <span>
          {errorCount > 0
            ? `${errorCount} Error${errorCount !== 1 ? 's' : ''}`
            : `${warningCount} Warning${warningCount !== 1 ? 's' : ''}`}
        </span>
      </button>

      {/* Expanded Error Panel */}
      {isExpanded && (
        <div
          style={{
            position: 'absolute',
            bottom: '70px',
            right: '0',
            width: '600px',
            maxWidth: '90vw',
            maxHeight: '70vh',
            backgroundColor: '#1a1f3a',
            border: '2px solid #ef4444',
            borderRadius: '12px',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '16px',
              backgroundColor: '#0f1629',
              borderBottom: '1px solid #ef4444',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
            }}
          >
            <h3
              style={{
                margin: '0',
                fontSize: '16px',
                fontWeight: 'bold',
                color: '#ffffff',
              }}
            >
              🚨 Error Logs {showAllLogs && '(All Stored)'}
            </h3>
            <button
              onClick={() => setIsExpanded(false)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#ffffff',
                cursor: 'pointer',
                fontSize: '20px',
                padding: '0',
                width: '24px',
                height: '24px',
              }}
            >
              ×
            </button>
          </div>

          {/* Error List */}
          <div
            style={{
              flex: '1',
              overflowY: 'auto',
              padding: '16px',
              backgroundColor: '#0a0e27',
            }}
          >
            {displayLogs.length === 0 ? (
              <div style={{ color: '#888888', textAlign: 'center', padding: '20px' }}>
                No errors logged
              </div>
            ) : (
              displayLogs.map((log, index) => (
                <div
                  key={`${log.timestamp}-${index}`}
                  style={{
                    marginBottom: '12px',
                    padding: '12px',
                    backgroundColor:
                      log.type === 'error'
                        ? '#2d1a1a'
                        : log.type === 'warning'
                          ? '#2d2a1a'
                          : '#1a2d1a',
                    border: `1px solid ${
                      log.type === 'error'
                        ? '#ef4444'
                        : log.type === 'warning'
                          ? '#f59e0b'
                          : '#22c55e'
                    }`,
                    borderRadius: '6px',
                  }}
                >
                  {/* Header */}
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: '8px',
                      marginBottom: '8px',
                    }}
                  >
                    <span style={{ fontSize: '16px' }}>
                      {log.type === 'error' ? '❌' : log.type === 'warning' ? '⚠️' : 'ℹ️'}
                    </span>
                    <div style={{ flex: '1' }}>
                      <div
                        style={{
                          fontWeight: 'bold',
                          color: '#ffffff',
                          marginBottom: '4px',
                        }}
                      >
                        {log.source}
                      </div>
                      <div style={{ color: '#cccccc', fontSize: '13px' }}>{log.message}</div>
                    </div>
                  </div>

                  {/* Timestamp */}
                  <div style={{ color: '#666666', fontSize: '11px', marginBottom: '8px' }}>
                    {new Date(log.timestamp).toLocaleString()}
                  </div>

                  {/* Stack Trace */}
                  {log.stack && (
                    <details style={{ marginTop: '8px' }}>
                      <summary
                        style={{
                          color: '#888888',
                          cursor: 'pointer',
                          fontSize: '12px',
                          marginBottom: '4px',
                        }}
                      >
                        Stack Trace
                      </summary>
                      <pre
                        style={{
                          margin: '8px 0 0 0',
                          padding: '8px',
                          backgroundColor: '#0a0e27',
                          border: '1px solid #333333',
                          borderRadius: '4px',
                          color: '#888888',
                          fontSize: '11px',
                          fontFamily: 'monospace',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-all',
                          maxHeight: '150px',
                          overflowY: 'auto',
                        }}
                      >
                        {log.stack}
                      </pre>
                    </details>
                  )}

                  {/* Context */}
                  {log.context && (
                    <details style={{ marginTop: '8px' }}>
                      <summary
                        style={{
                          color: '#888888',
                          cursor: 'pointer',
                          fontSize: '12px',
                          marginBottom: '4px',
                        }}
                      >
                        Context
                      </summary>
                      <pre
                        style={{
                          margin: '8px 0 0 0',
                          padding: '8px',
                          backgroundColor: '#0a0e27',
                          border: '1px solid #333333',
                          borderRadius: '4px',
                          color: '#888888',
                          fontSize: '11px',
                          fontFamily: 'monospace',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-all',
                          maxHeight: '150px',
                          overflowY: 'auto',
                        }}
                      >
                        {JSON.stringify(log.context, null, 2)}
                      </pre>
                    </details>
                  )}
                </div>
              ))
            )}
          </div>

          {/* Footer Actions */}
          <div
            style={{
              padding: '16px',
              backgroundColor: '#0f1629',
              borderTop: '1px solid #333333',
              display: 'flex',
              gap: '8px',
              flexWrap: 'wrap',
            }}
          >
            <button
              onClick={handleRestart}
              style={{
                flex: '1',
                minWidth: '120px',
                padding: '10px 16px',
                backgroundColor: '#22c55e',
                color: '#000000',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontWeight: 'bold',
                fontSize: '13px',
              }}
            >
              🔄 Restart App
            </button>
            <button
              onClick={handleCopyAll}
              style={{
                flex: '1',
                minWidth: '120px',
                padding: '10px 16px',
                backgroundColor: '#3b82f6',
                color: '#ffffff',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontWeight: 'bold',
                fontSize: '13px',
              }}
            >
              📋 Copy Report
            </button>
            <button
              onClick={handleToggleLogs}
              style={{
                flex: '1',
                minWidth: '120px',
                padding: '10px 16px',
                backgroundColor: '#8b5cf6',
                color: '#ffffff',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontWeight: 'bold',
                fontSize: '13px',
              }}
            >
              {showAllLogs ? '📝 Recent' : '📚 All Stored'}
            </button>
            <button
              onClick={handleClear}
              style={{
                flex: '1',
                minWidth: '120px',
                padding: '10px 16px',
                backgroundColor: '#ef4444',
                color: '#ffffff',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
                fontWeight: 'bold',
                fontSize: '13px',
              }}
            >
              🗑️ Clear All
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
