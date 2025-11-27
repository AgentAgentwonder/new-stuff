import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { DevErrorDisplay } from './components/DevErrorDisplay';
import { FailsafeErrorOverlay } from './components/FailsafeErrorOverlay';
import { errorLogger } from './utils/errorLogger';
import './styles/globals.css';

// CRITICAL: Safe global error handlers with recursion prevention
// These handlers must NEVER throw errors themselves to prevent infinite loops
let errorHandlerActive = false;
let promiseRejectionHandlerActive = false;

window.addEventListener('error', (event: ErrorEvent) => {
  // Prevent recursive error handling
  if (errorHandlerActive) {
    return;
  }

  try {
    errorHandlerActive = true;
    // Don't log internal errorLogger errors to prevent recursion
    if (event.message && !event.message.includes('errorLogger')) {
      errorLogger.error(event.message, 'Global Error Handler', event.error, {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
      });
    }
  } catch (_e) {
    // Silently ignore - do not call errorLogger as it could cause infinite recursion
  } finally {
    errorHandlerActive = false;
  }
});

// Set up global handler for unhandled promise rejections
window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
  // Prevent recursive rejection handling
  if (promiseRejectionHandlerActive) {
    return;
  }

  try {
    promiseRejectionHandlerActive = true;
    const reason = event.reason;
    if (reason instanceof Error) {
      errorLogger.error(
        `Unhandled Promise Rejection: ${reason.message}`,
        'Global Promise Handler',
        reason
      );
    } else {
      const reasonStr = String(reason);
      // Don't log internal errorLogger errors to prevent recursion
      if (!reasonStr.includes('errorLogger')) {
        errorLogger.error(`Unhandled Promise Rejection: ${reasonStr}`, 'Global Promise Handler');
      }
    }
  } catch (_e) {
    // Silently ignore - do not call errorLogger as it could cause infinite recursion
  } finally {
    promiseRejectionHandlerActive = false;
  }
});

// Log app initialization start
errorLogger.info('Application initializing', 'main.tsx');

try {
  errorLogger.info('Creating React root', 'main.tsx');
  const root = document.getElementById('root');

  if (!root) {
    throw new Error('Root element not found in DOM');
  }

  errorLogger.info('Root element found, creating ReactDOM root', 'main.tsx');

  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <DevErrorDisplay>
        <App />
      </DevErrorDisplay>
      <FailsafeErrorOverlay />
    </React.StrictMode>
  );

  errorLogger.info('React app mounted successfully', 'main.tsx');
} catch (error) {
  const errorMessage = error instanceof Error ? error.message : String(error);
  errorLogger.error(
    `Failed to mount React app: ${errorMessage}`,
    'main.tsx',
    error instanceof Error ? error : undefined
  );

  // Display error on screen with enhanced UI
  const root = document.getElementById('root');
  if (root) {
    const storedLogs = sessionStorage.getItem('eclipse_error_logs');
    const previousErrors = storedLogs ? JSON.parse(storedLogs).slice(-5) : [];

    root.innerHTML = `
      <div style="
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100vh;
        background-color: #0a0e27;
        color: #ffffff;
        padding: 20px;
        font-family: system-ui, -apple-system, sans-serif;
      ">
        <div style="
          max-width: 700px;
          width: 100%;
          background-color: #1a1f3a;
          border: 2px solid #ff6b6b;
          border-radius: 12px;
          padding: 30px;
          box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
        ">
          <div style="text-align: center; margin-bottom: 20px;">
            <div style="font-size: 48px; margin-bottom: 10px;">🚨</div>
            <h1 style="margin: 0 0 10px 0; color: #ff6b6b; font-size: 24px;">Application Failed to Start</h1>
            <p style="margin: 0; color: #cccccc; font-size: 16px;">${errorMessage}</p>
          </div>
          
          <div style="
            background-color: #0a0e27;
            border: 1px solid #444;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 20px;
          ">
            <div style="color: #888; font-size: 12px; margin-bottom: 8px; font-weight: bold;">
              STACK TRACE:
            </div>
            <pre style="
              margin: 0;
              color: #888;
              font-size: 11px;
              font-family: monospace;
              overflow-x: auto;
              max-height: 200px;
              overflow-y: auto;
              white-space: pre-wrap;
              word-break: break-all;
            ">${error instanceof Error ? error.stack || 'No stack trace available' : String(error)}</pre>
          </div>

          ${
            previousErrors.length > 0
              ? `
          <details style="margin-bottom: 20px;">
            <summary style="
              cursor: pointer;
              color: #f59e0b;
              font-weight: bold;
              margin-bottom: 10px;
              font-size: 14px;
            ">
              ⚠️ Previous Errors (${previousErrors.length})
            </summary>
            <div style="
              background-color: #0a0e27;
              border: 1px solid #444;
              border-radius: 8px;
              padding: 12px;
              max-height: 200px;
              overflow-y: auto;
            ">
              ${previousErrors
                .map(
                  (log: any) => `
                <div style="margin-bottom: 12px; padding-bottom: 12px; border-bottom: 1px solid #333;">
                  <div style="color: #ffffff; font-size: 12px; font-weight: bold; margin-bottom: 4px;">
                    ${log.source}
                  </div>
                  <div style="color: #cccccc; font-size: 11px; margin-bottom: 4px;">
                    ${log.message}
                  </div>
                  <div style="color: #666; font-size: 10px;">
                    ${new Date(log.timestamp).toLocaleString()}
                  </div>
                </div>
              `
                )
                .join('')}
            </div>
          </details>
          `
              : ''
          }
          
          <div style="
            display: flex;
            gap: 12px;
            margin-bottom: 20px;
            flex-wrap: wrap;
          ">
            <button onclick="location.reload()" style="
              flex: 1;
              min-width: 120px;
              padding: 14px 24px;
              background-color: #22c55e;
              color: #000;
              border: none;
              border-radius: 8px;
              cursor: pointer;
              font-weight: bold;
              font-size: 14px;
              box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
            ">🔄 Restart App</button>
            
            <button onclick="
              const logs = localStorage.getItem('eclipse_error_logs');
              const report = '=== ECLIPSE MARKET PRO ERROR REPORT ===' + String.fromCharCode(10) +
                'Timestamp: ' + new Date().toISOString() + String.fromCharCode(10) +
                'User Agent: ' + navigator.userAgent + String.fromCharCode(10) +
                'URL: ' + window.location.href + String.fromCharCode(10) +
                String.fromCharCode(10) +
                '=== CURRENT ERROR ===' + String.fromCharCode(10) +
                '${errorMessage}' + String.fromCharCode(10) +
                String.fromCharCode(10) +
                '${error instanceof Error ? (error.stack || '').replace(/\n/g, String.fromCharCode(10)) : String(error)}' +
                String.fromCharCode(10) + String.fromCharCode(10) +
                '=== ALL STORED LOGS ===' + String.fromCharCode(10) +
                (logs || 'No stored logs');
              navigator.clipboard.writeText(report).then(() => {
                alert('Full error report copied to clipboard!');
              }).catch(() => {
                alert('Failed to copy. Check console for full report.');
                console.log(report);
              });
            " style="
              flex: 1;
              min-width: 120px;
              padding: 14px 24px;
              background-color: #3b82f6;
              color: #fff;
              border: none;
              border-radius: 8px;
              cursor: pointer;
              font-weight: bold;
              font-size: 14px;
              box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
            ">📋 Copy Full Report</button>
            
            <button onclick="
              if (confirm('Clear all error logs? This cannot be undone.')) {
                localStorage.removeItem('eclipse_error_logs');
                alert('Error logs cleared. Restarting...');
                location.reload();
              }
            " style="
              flex: 1;
              min-width: 120px;
              padding: 14px 24px;
              background-color: #ef4444;
              color: #fff;
              border: none;
              border-radius: 8px;
              cursor: pointer;
              font-weight: bold;
              font-size: 14px;
              box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
            ">🗑️ Clear & Restart</button>
          </div>

          <div style="
            background-color: #0f1629;
            border: 1px solid #333;
            border-radius: 6px;
            padding: 12px;
            text-align: center;
          ">
            <div style="color: #888; font-size: 12px; margin-bottom: 8px;">
              💡 <strong>Troubleshooting Tips:</strong>
            </div>
            <ul style="
              color: #888;
              font-size: 11px;
              text-align: left;
              margin: 0;
              padding-left: 20px;
              line-height: 1.6;
            ">
              <li>Clear browser cache and reload (Ctrl+Shift+R or Cmd+Shift+R)</li>
              <li>Check if browser console (F12) shows additional errors</li>
              <li>Verify internet connection for external dependencies</li>
              <li>Try disabling browser extensions</li>
              <li>Copy the error report and contact support if issue persists</li>
            </ul>
          </div>
        </div>
      </div>
    `;
  }
}
