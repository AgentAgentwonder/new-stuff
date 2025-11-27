import { errorLogger } from './errorLogger';

export interface TauriHealthStatus {
  status: string;
  timestamp: string;
  version: string;
  backend_initialized: boolean;
}

/**
 * Check if Tauri backend is available and responding
 * Returns health status if available, null if not
 */
export async function checkTauriHealth(): Promise<TauriHealthStatus | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const health = await invoke<TauriHealthStatus>('check_tauri_health');
    errorLogger.info(`Tauri backend healthy (v${health.version})`, 'TauriHealthCheck');
    return health;
  } catch (error) {
    errorLogger.error(
      'Tauri backend not responding',
      'TauriHealthCheck',
      error instanceof Error ? error : undefined,
      {
        message: error instanceof Error ? error.message : String(error),
      }
    );
    return null;
  }
}

/**
 * Wait for Tauri backend to become available
 * Returns true if backend is available within timeout, false otherwise
 */
export async function waitForTauriBackend(
  timeoutMs: number = 5000,
  retryIntervalMs: number = 500
): Promise<boolean> {
  const startTime = Date.now();

  while (Date.now() - startTime < timeoutMs) {
    const health = await checkTauriHealth();
    if (health) {
      return true;
    }
    await new Promise(resolve => setTimeout(resolve, retryIntervalMs));
  }

  errorLogger.error(`Tauri backend did not respond within ${timeoutMs}ms`, 'TauriHealthCheck');
  return false;
}

/**
 * Check if we're running in Tauri environment
 */
export function isTauriEnvironment(): boolean {
  return '__TAURI__' in window;
}
