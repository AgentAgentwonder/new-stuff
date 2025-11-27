import { useState, useCallback, useEffect, useRef } from 'react';
import { useUIStore } from '../store/uiStore';
import { errorLogger } from '@/utils/errorLogger';
import type { ApiResponse, TauriError } from '../lib/tauri/types';

// Generic options for the hook
interface UseTauriCommandOptions<T> {
  onSuccess?: (data: T) => void;
  onError?: (error: TauriError) => void;
  showToastOnError?: boolean;
  loadingId?: string;
  loadingMessage?: string;
  immediate?: boolean;
  dependencies?: any[];
}

// Return type for the hook
interface UseTauriCommandReturn<T, A extends any[]> {
  data: T | null;
  isLoading: boolean;
  error: TauriError | null;
  execute: (...args: A) => Promise<ApiResponse<T>>;
  reset: () => void;
}

/**
 * Generic hook for executing Tauri commands with consistent loading/error handling
 *
 * @param commandFn - Function that returns a Promise<ApiResponse<T>>
 * @param options - Configuration options
 * @returns Hook state and execute function
 */
export function useTauriCommand<T, A extends any[] = []>(
  commandFn: (...args: A) => Promise<ApiResponse<T>>,
  options: UseTauriCommandOptions<T> = {}
): UseTauriCommandReturn<T, A> {
  const {
    onSuccess,
    onError,
    showToastOnError = true,
    loadingId,
    loadingMessage,
    immediate = false,
    dependencies = [],
  } = options;

  const [data, setData] = useState<T | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<TauriError | null>(null);

  const uiSelector = useCallback(
    (state: ReturnType<typeof useUIStore.getState>) => ({
      setLoading: state.setLoading,
      addToast: state.addToast,
    }),
    []
  );
  const { setLoading, addToast } = useUIStore(uiSelector);
  const mountedRef = useRef(true);
  const argsRef = useRef<A>();

  // Reset state
  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setIsLoading(false);
  }, []);

  // Execute command
  const execute = useCallback(
    async (...args: A): Promise<ApiResponse<T>> => {
      argsRef.current = args;
      setIsLoading(true);
      setError(null);

      // Set loading state in UI store if loadingId provided
      if (loadingId) {
        setLoading(loadingId, true, loadingMessage);
      }

      try {
        errorLogger.info(`Executing command: ${loadingMessage || 'Operation'}`, 'useTauriCommand', {
          loadingId,
          args: JSON.stringify(args).substring(0, 100),
        });

        const response = await commandFn(...args);

        if (!mountedRef.current) {
          return response;
        }

        if (response.success) {
          setData(response.data);
          setError(null);
          errorLogger.info(
            `Command completed successfully: ${loadingMessage || 'Operation'}`,
            'useTauriCommand'
          );
          onSuccess?.(response.data);
        } else {
          setError(response.error || { message: 'Unknown error' });
          onError?.(response.error || { message: 'Unknown error' });
          errorLogger.error(
            `Command failed: ${response.error?.message || 'Unknown error'}`,
            'useTauriCommand',
            undefined,
            { loadingId, error: response.error }
          );

          // Show toast error if enabled
          if (showToastOnError && response.error) {
            addToast({
              type: 'error',
              title: 'Operation Failed',
              message: response.error.message,
            });
          }
        }

        return response;
      } catch (err) {
        const errorObj: TauriError = {
          message: err instanceof Error ? err.message : 'Unexpected error occurred',
        };

        errorLogger.error(
          `Command execution error: ${errorObj.message}`,
          'useTauriCommand',
          err instanceof Error ? err : undefined,
          { loadingId }
        );

        if (mountedRef.current) {
          setError(errorObj);
          onError?.(errorObj);

          // Show toast error if enabled
          if (showToastOnError) {
            addToast({
              type: 'error',
              title: 'Operation Failed',
              message: errorObj.message,
            });
          }
        }

        return {
          data: null as any,
          success: false,
          error: errorObj,
        };
      } finally {
        if (mountedRef.current) {
          setIsLoading(false);
        }

        // Clear loading state in UI store if loadingId provided
        if (loadingId) {
          setLoading(loadingId, false);
        }
      }
    },
    [
      commandFn,
      onSuccess,
      onError,
      showToastOnError,
      loadingId,
      loadingMessage,
      setLoading,
      addToast,
    ]
  );

  // Immediate execution if requested
  useEffect(() => {
    if (immediate && dependencies.length === 0) {
      execute();
    }
  }, [immediate, execute, ...dependencies]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
    };
  }, []);

  return {
    data,
    isLoading,
    error,
    execute,
    reset,
  };
}

/**
 * Hook for executing commands that return arrays, with additional array-specific utilities
 */
export function useTauriArrayCommand<T, A extends any[] = []>(
  commandFn: (...args: A) => Promise<ApiResponse<T[]>>,
  options: UseTauriCommandOptions<T[]> = {}
) {
  const result = useTauriCommand<T[], A>(commandFn, options);

  return {
    ...result,
    data: result.data || [],
    isEmpty: !result.isLoading && (!result.data || result.data.length === 0),
    count: result.data?.length || 0,
  };
}

/**
 * Hook for paginated data
 */
export function useTauriPaginatedCommand<T, A extends any[] = []>(
  commandFn: (...args: A) => Promise<ApiResponse<{ items: T[]; total: number; page: number }>>,
  options: UseTauriCommandOptions<{ items: T[]; total: number; page: number }> & {
    pageSize?: number;
  } = {}
) {
  const { pageSize = 20, ...commandOptions } = options;
  const [currentPage, setCurrentPage] = useState(1);

  const paginatedCommandFn = useCallback(
    (...args: A) => commandFn(...args, currentPage, pageSize),
    [commandFn, currentPage, pageSize]
  );

  const result = useTauriCommand(paginatedCommandFn, commandOptions);

  return {
    ...result,
    data: result.data?.items || [],
    total: result.data?.total || 0,
    page: result.data?.page || currentPage,
    pageSize,
    currentPage,
    setCurrentPage,
    hasNextPage: (result.data?.total || 0) > currentPage * pageSize,
    hasPreviousPage: currentPage > 1,
    nextPage: () => setCurrentPage(prev => prev + 1),
    previousPage: () => setCurrentPage(prev => Math.max(1, prev - 1)),
    goToPage: (page: number) => setCurrentPage(Math.max(1, page)),
  };
}
