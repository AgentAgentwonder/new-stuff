import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import { moduleLogger } from '@/utils/moduleLogger';
import { AppErrorBoundary } from '@/components';

// Track component imports
moduleLogger.start('AppErrorBoundary');
moduleLogger.end('AppErrorBoundary');

moduleLogger.start('AccessibilityProvider');
import { AccessibilityProvider } from '@/components/providers/AccessibilityProvider';
moduleLogger.end('AccessibilityProvider');

moduleLogger.start('ClientLayout');
import ClientLayout from '@/layouts/ClientLayout';
moduleLogger.end('ClientLayout');

const LoadingFallback = () => <div style={{ padding: '20px', color: 'white' }}>Loading...</div>;

// Only load Dashboard for minimal test
const Dashboard = lazy(() => {
  moduleLogger.start('Dashboard');
  return import('@/pages/Dashboard').then(module => {
    moduleLogger.end('Dashboard');
    return module;
  }).catch(error => {
    moduleLogger.error('Dashboard', error);
    return { default: () => <LoadingFallback /> };
  });
});

function AppTest() {
  return (
    <AppErrorBoundary>
      <AccessibilityProvider>
        <HashRouter>
          <ClientLayout>
            <Suspense fallback={<LoadingFallback />}>
              <Routes>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                <Route path="/dashboard" element={<Dashboard />} />
                <Route path="*" element={<Navigate to="/dashboard" replace />} />
              </Routes>
            </Suspense>
          </ClientLayout>
        </HashRouter>
      </AccessibilityProvider>
    </AppErrorBoundary>
  );
}

export default AppTest;
