import React, { useCallback, useState, useEffect } from 'react';
import { useUIStore } from '../store/uiStore';

/**
 * Hook for managing developer console functionality
 * Only works in development builds (when __DEV__ is true)
 */
export function useDevConsole() {
  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      devConsoleOpen: state.devConsoleOpen,
      setDevConsoleOpen: state.setDevConsoleOpen,
    }),
    []
  );
  const { devConsoleOpen, setDevConsoleOpen } = useUIStore(uiSelector);
  const [appWindow, setAppWindow] = useState<any>(null);

  useEffect(() => {
    import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
      setAppWindow(getCurrentWindow());
    });
  }, []);

  const toggleDevConsole = useCallback(async () => {
    // Only allow in development builds
    if (process.env.NODE_ENV !== 'development') {
      console.warn('Dev console is only available in development builds');
      return;
    }

    if (!appWindow) return;

    try {
      if (devConsoleOpen) {
        // Close devtools
        await appWindow.closeDevtools();
        setDevConsoleOpen(false);
      } else {
        // Open devtools
        await appWindow.openDevtools();
        setDevConsoleOpen(true);
      }
    } catch (error) {
      console.error('Failed to toggle dev console:', error);
    }
  }, [devConsoleOpen, setDevConsoleOpen, appWindow]);

  const openDevConsole = useCallback(async () => {
    if (process.env.NODE_ENV !== 'development') {
      console.warn('Dev console is only available in development builds');
      return;
    }

    if (!appWindow) return;

    try {
      if (!devConsoleOpen) {
        await appWindow.openDevtools();
        setDevConsoleOpen(true);
      }
    } catch (error) {
      console.error('Failed to open dev console:', error);
    }
  }, [devConsoleOpen, setDevConsoleOpen, appWindow]);

  const closeDevConsole = useCallback(async () => {
    if (process.env.NODE_ENV !== 'development') {
      return;
    }

    if (!appWindow) return;

    try {
      if (devConsoleOpen) {
        await appWindow.closeDevtools();
        setDevConsoleOpen(false);
      }
    } catch (error) {
      console.error('Failed to close dev console:', error);
    }
  }, [devConsoleOpen, setDevConsoleOpen, appWindow]);

  return {
    isDevConsoleOpen: devConsoleOpen,
    toggleDevConsole,
    openDevConsole,
    closeDevConsole,
    // Convenience method to check if dev console is available
    isDevConsoleAvailable: process.env.NODE_ENV === 'development',
  };
}

/**
 * Hook for keyboard shortcuts related to dev console
 */
export function useDevConsoleShortcuts() {
  const { toggleDevConsole, isDevConsoleAvailable } = useDevConsole();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // F12 or Ctrl+Shift+I (Cmd+Opt+I on Mac) to toggle dev console
      if (
        isDevConsoleAvailable &&
        (event.key === 'F12' ||
          (event.ctrlKey && event.shiftKey && event.key === 'I') ||
          (event.metaKey && event.altKey && event.key === 'I'))
      ) {
        event.preventDefault();
        toggleDevConsole();
      }
    },
    [isDevConsoleAvailable, toggleDevConsole]
  );

  return {
    handleKeyDown,
  };
}

/**
 * Hook that automatically sets up dev console keyboard shortcuts
 */
export function useDevConsoleAutoSetup() {
  const { handleKeyDown } = useDevConsoleShortcuts();

  // Set up keyboard event listener
  React.useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);
}
