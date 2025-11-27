import React from 'react';
import ReactDOM from 'react-dom/client';
import { moduleLogger } from '@/utils/moduleLogger';
import { DevErrorDisplay } from '@/components/DevErrorDisplay';
import { FailsafeErrorOverlay } from '@/components/FailsafeErrorOverlay';
import { errorLogger } from '@/utils/errorLogger';
import './styles/globals.css';

// Start logging immediately
moduleLogger.start('main.test.tsx');
console.log('[MAIN] Application starting...');

// CRITICAL: Safe global error handlers with recursion prevention
let errorHandlerActive = false;
let promiseRejectionHandlerActive = false;

window.addEventListener('error', (event: ErrorEvent) => {
  if (errorHandlerActive) return;
  try {
    errorHandlerActive = true;
    if (event.message && !event.message.includes('errorLogger')) {
      errorLogger.error(event.message, 'Global Error Handler', event.error, {
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
      });
    }
  } catch (_e) {
    console.error('Error in global error handler:', _e);
  } finally {
    errorHandlerActive = false;
  }
});

window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
  if (promiseRejectionHandlerActive) return;
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
      if (!reasonStr.includes('errorLogger')) {
        errorLogger.error(`Unhandled Promise Rejection: ${reasonStr}`, 'Global Promise Handler');
      }
    }
  } catch (_e) {
    console.error('Error in rejection handler:', _e);
  } finally {
    promiseRejectionHandlerActive = false;
  }
});

errorLogger.info('Application initializing (TEST MODE)', 'main.test.tsx');

try {
  moduleLogger.start('Importing AppTest');
  
  // Import AppTest dynamically to track when it loads
  import('./App.test').then(module => {
    moduleLogger.end('Importing AppTest');
    
    errorLogger.info('Creating React root', 'main.test.tsx');
    const root = document.getElementById('root');

    if (!root) {
      throw new Error('Root element not found in DOM');
    }

    errorLogger.info('Root element found, creating ReactDOM root', 'main.test.tsx');

    ReactDOM.createRoot(root).render(
      <React.StrictMode>
        <DevErrorDisplay>
          <module.default />
        </DevErrorDisplay>
        <FailsafeErrorOverlay />
      </React.StrictMode>
    );

    errorLogger.info('React app mounted successfully (TEST MODE)', 'main.test.tsx');
    moduleLogger.end('main.test.tsx');
    
    // Print the module load report
    setTimeout(() => {
      moduleLogger.printReport();
    }, 1000);
  }).catch(error => {
    moduleLogger.error('Importing AppTest', error);
    throw error;
  });
} catch (error) {
  moduleLogger.error('main.test.tsx', error);
  
  const errorMessage = error instanceof Error ? error.message : String(error);
  errorLogger.error(
    `Failed to mount React app: ${errorMessage}`,
    'main.test.tsx',
    error instanceof Error ? error : undefined
  );

  // Display error on screen
  const root = document.getElementById('root');
  if (root) {
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
            <h1 style="margin: 0 0 10px 0; color: #ff6b6b; font-size: 24px;">TEST App Failed to Start</h1>
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

          <div style="
            background-color: #0a0e27;
            border: 1px solid #444;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 20px;
          ">
            <div style="color: #888; font-size: 12px; margin-bottom: 8px; font-weight: bold;">
              MODULE LOAD REPORT:
            </div>
            <pre style="
              margin: 0;
              color: #888;
              font-size: 10px;
              font-family: monospace;
              overflow-x: auto;
              max-height: 300px;
              overflow-y: auto;
              white-space: pre-wrap;
              word-break: break-all;
            ">${moduleLogger.getReport()}</pre>
          </div>

          <button onclick="location.reload()" style="
            width: 100%;
            padding: 14px 24px;
            background-color: #22c55e;
            color: #000;
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: bold;
            font-size: 14px;
            box-shadow: 0 2px 8px rgba(34, 197, 94, 0.3);
          ">🔄 Restart Test</button>
        </div>
      </div>
    `;
  }
}
