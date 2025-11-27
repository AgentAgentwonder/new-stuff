import { useCallback, useRef } from 'react';

/**
 * Hook that returns a memoized callback that maintains stable identity
 * across re-renders, even if the callback function changes.
 *
 * This is useful for:
 * - Preventing infinite loops in useEffect
 * - Stable references for event handlers
 * - Optimizing performance by avoiding unnecessary re-renders
 *
 * @param fn - The callback function to stabilize
 * @returns A stable callback function
 */
export function useStableCallback<T extends (...args: any[]) => any>(fn: T): T {
  const fnRef = useRef<T>(fn);

  // Update the ref when the function changes
  fnRef.current = fn;

  // Return a stable callback that calls the latest function
  return useCallback((...args: Parameters<T>) => {
    return fnRef.current(...args);
  }, []) as T;
}

/**
 * Hook that returns a memoized callback with stable identity that
 * also maintains the latest context/values when called.
 *
 * Unlike useCallback which can capture stale values from previous renders,
 * this hook ensures the callback always has access to the latest values.
 *
 * @param fn - The callback function to stabilize
 * @param deps - Dependency array (optional, similar to useCallback)
 * @returns A stable callback function with latest values
 */
export function useLatestCallback<T extends (...args: any[]) => any>(
  fn: T,
  deps: React.DependencyList = []
): T {
  const fnRef = useRef<T>(fn);
  const depsRef = useRef(deps);

  // Update the ref when dependencies change
  if (!depsRef.current.every((dep, i) => dep === deps[i])) {
    fnRef.current = fn;
    depsRef.current = deps;
  }

  // Return a stable callback that calls the latest function
  return useCallback((...args: Parameters<T>) => {
    return fnRef.current(...args);
  }, []) as T;
}

/**
 * Hook that creates a debounced callback with stable identity.
 * The callback will only be executed after the specified delay
 * has passed without the callback being called again.
 *
 * @param fn - The callback function to debounce
 * @param delay - Delay in milliseconds
 * @param deps - Dependency array
 * @returns A debounced callback function
 */
export function useDebouncedCallback<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
  deps: React.DependencyList = []
): T {
  const timeoutRef = useRef<NodeJS.Timeout>();
  const fnRef = useRef<T>(fn);
  const depsRef = useRef(deps);

  // Update refs when dependencies change
  if (!depsRef.current.every((dep, i) => dep === deps[i])) {
    fnRef.current = fn;
    depsRef.current = deps;
  }

  return useCallback((...args: Parameters<T>) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = setTimeout(() => {
      fnRef.current(...args);
    }, delay);
  }, []) as T;
}

/**
 * Hook that creates a throttled callback with stable identity.
 * The callback will only be executed once per specified time period.
 *
 * @param fn - The callback function to throttle
 * @param delay - Delay in milliseconds
 * @param deps - Dependency array
 * @returns A throttled callback function
 */
export function useThrottledCallback<T extends (...args: any[]) => any>(
  fn: T,
  delay: number,
  deps: React.DependencyList = []
): T {
  const lastCallRef = useRef<number>(0);
  const fnRef = useRef<T>(fn);
  const depsRef = useRef(deps);

  // Update refs when dependencies change
  if (!depsRef.current.every((dep, i) => dep === deps[i])) {
    fnRef.current = fn;
    depsRef.current = deps;
  }

  return useCallback((...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCallRef.current >= delay) {
      lastCallRef.current = now;
      fnRef.current(...args);
    }
  }, []) as T;
}
