import React, { useEffect, useState } from 'react';
import { ErrorBoundary } from './src/components/common/ErrorBoundary';

// Minimal test app to isolate startup crash
function MinimalApp() {
  const [logs, setLogs] = useState<string[]>([]);
  const [mounted, setMounted] = useState(false);

  const addLog = (message: string) => {
    const timestamp = new Date().toISOString();
    setLogs(prev => [...prev, `[${timestamp}] ${message}`]);
    console.log(`[MinimalApp] ${message}`);
  };

  useEffect(() => {
    addLog('MinimalApp component mounted');
    setMounted(true);

    // Test basic Tauri API
    const testTauriAPI = async () => {
      try {
        addLog('Testing Tauri API...');
        const { invoke } = await import('@tauri-apps/api/core');
        addLog('Tauri invoke imported successfully');

        // Test a simple command that should exist
        const result = await invoke('biometric_get_status');
        addLog(`Tauri command result: ${JSON.stringify(result)}`);
      } catch (error) {
        addLog(`Tauri API test failed: ${error}`);
      }
    };

    testTauriAPI();

    // Test Zustand stores
    const testStores = async () => {
      try {
        addLog('Testing Zustand stores...');
        const { useThemeStore } = await import('./src/store/themeStore');
        const theme = useThemeStore.getState();
        addLog(`Theme store loaded: ${theme.currentTheme.name}`);

        const { useAccessibilityStore } = await import('./src/store/accessibilityStore');
        const accessibility = useAccessibilityStore.getState();
        addLog(`Accessibility store loaded: fontScale=${accessibility.fontScale}`);
      } catch (error) {
        addLog(`Store test failed: ${error}`);
      }
    };

    testStores();

    return () => {
      addLog('MinimalApp component unmounting');
    };
  }, []);

  if (!mounted) {
    return (
      <div style={{ padding: '20px', fontFamily: 'monospace' }}>
        <h2>Initializing Minimal App...</h2>
      </div>
    );
  }

  return (
    <div
      style={{
        padding: '20px',
        fontFamily: 'monospace',
        backgroundColor: '#1a1a1a',
        color: '#fff',
        minHeight: '100vh',
      }}
    >
      <h1>Eclipse Market Pro - Minimal Test App</h1>
      <p>This is a minimal version to isolate startup issues.</p>

      <div
        style={{
          marginTop: '20px',
          padding: '10px',
          backgroundColor: '#2a2a2a',
          borderRadius: '5px',
        }}
      >
        <h3>Startup Logs:</h3>
        <div style={{ maxHeight: '300px', overflow: 'auto', fontSize: '12px' }}>
          {logs.map((log, index) => (
            <div key={index} style={{ marginBottom: '5px' }}>
              {log}
            </div>
          ))}
        </div>
      </div>

      <div style={{ marginTop: '20px' }}>
        <button
          onClick={() => window.location.reload()}
          style={{
            padding: '10px 20px',
            marginRight: '10px',
            backgroundColor: '#007acc',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
          }}
        >
          Reload
        </button>
        <button
          onClick={() => {
            import('./src/App')
              .then(() => {
                addLog('Full App module loaded successfully');
              })
              .catch(error => {
                addLog(`Failed to load full App: ${error}`);
              });
          }}
          style={{
            padding: '10px 20px',
            backgroundColor: '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '5px',
          }}
        >
          Test Full App Import
        </button>
      </div>
    </div>
  );
}

// Wrapper with error boundary
function AppWithBoundary() {
  return (
    <ErrorBoundary>
      <MinimalApp />
    </ErrorBoundary>
  );
}

export default AppWithBoundary;
