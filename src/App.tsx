import { useEffect } from 'react';
import { AccessibilityProvider } from '@/components/providers/AccessibilityProvider';
import Dashboard from '@/components/Dashboard/Dashboard';
import './styles/globals.css';

function App() {
  useEffect(() => {
    // Add v0 theme class for the dashboard styling
    document.documentElement.classList.add('v0-theme');
    document.documentElement.setAttribute('data-theme', 'v0');
    
    return () => {
      // Clean up theme on unmount
      document.documentElement.classList.remove('v0-theme');
      document.documentElement.removeAttribute('data-theme');
    };
  }, []);

  return (
    <AccessibilityProvider>
      <Dashboard />
    </AccessibilityProvider>
  );
}

export default App;