import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App.tsx';

console.log('[main] Starting minimal app initialization');

const root = ReactDOM.createRoot(document.getElementById('root')!);

try {
  console.log('[main] Rendering minimal app');
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
  console.log('[main] Minimal app rendered successfully');
} catch (error) {
  console.error('[main] Failed to render minimal app:', error);

  // Fallback rendering
  root.render(
    <div
      style={{
        padding: '20px',
        fontFamily: 'monospace',
        backgroundColor: '#1a1a1a',
        color: '#fff',
        minHeight: '100vh',
      }}
    >
      <h1>Critical Error During Startup</h1>
      <p>The application failed to initialize. Error details:</p>
      <pre style={{ backgroundColor: '#2a2a2a', padding: '10px', borderRadius: '5px' }}>
        {error instanceof Error ? error.stack : String(error)}
      </pre>
      <button onClick={() => window.location.reload()}>Reload Application</button>
    </div>
  );
}
