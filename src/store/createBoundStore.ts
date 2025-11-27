import {
  createStore as createZustandStore,
  useStore as useZustandStore,
  type StateCreator,
  type StoreApi,
} from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

// Zustand v5: shallow comparator function for object equality
// Compares objects by reference and shallow property comparison
function useShallow<T>(a: T, b: T): boolean {
  if (Object.is(a, b)) return true;
  if (!a || !b || typeof a !== 'object' || typeof b !== 'object') {
    return false;
  }
  const keysA = Object.keys(a as Record<string, unknown>);
  const keysB = Object.keys(b as Record<string, unknown>);
  if (keysA.length !== keysB.length) return false;
  return keysA.every(key => Object.is((a as any)[key], (b as any)[key]));
}

export { useShallow };

export type CreateStoreResult<T> = {
  store: StoreApi<T>;
  useStore: {
    (): T;
    <U>(selector: (state: T) => U, equalityFn?: (a: U, b: U) => boolean): U;
    getState: () => T;
  };
  getState: () => T;
  setState: (partial: Partial<T> | ((state: T) => Partial<T>)) => void;
  subscribe: (listener: (state: T, prevState: T) => void) => () => void;
};

/**
 * Creates a bound Zustand store with subscribeWithSelector middleware enabled by default.
 * This ensures all stores can handle selective subscriptions and prevents stale snapshots.
 *
 * @param initializer - State creator function (set, get, api) => state
 * @returns Store result with typed hooks
 *
 * @example
 * ```typescript
 * const storeResult = createBoundStore<MyState>((set, get) => ({
 *   count: 0,
 *   increment: () => set(state => ({ count: state.count + 1 })),
 * }));
 *
 * export const useMyStore = storeResult.useStore;
 * ```
 */
export function createBoundStore<T>(
  initializer: StateCreator<T, [['zustand/subscribeWithSelector', never]], [], T>
): CreateStoreResult<T> {
  // Wrap initializer with subscribeWithSelector middleware
  const store = createZustandStore<T>()(subscribeWithSelector(initializer));

  const useStore: any = <U = T>(
    selector?: (state: T) => U,
    equalityFn?: (a: U, b: U) => boolean
  ) => {
    return useZustandStore(store, selector as any, equalityFn as any);
  };

  // Add getState to useStore hook
  useStore.getState = store.getState;

  return {
    store,
    useStore,
    getState: store.getState,
    setState: store.setState,
    subscribe: store.subscribe,
  };
}

/**
 * Creates a bound Zustand store with custom middleware.
 * Use this when you need to compose additional middleware like persist or devtools.
 * subscribeWithSelector is NOT automatically applied - you must include it if needed.
 *
 * @param creator - Middleware-wrapped state creator
 * @returns Store result with typed hooks
 *
 * @example
 * ```typescript
 * import { persist, createJSONStorage } from 'zustand/middleware';
 * import { getPersistentStorage } from './storage';
 *
 * const storeResult = createBoundStoreWithMiddleware<MyState>()(
 *   subscribeWithSelector(
 *     persist(
 *       (set, get) => ({
 *         theme: 'dark',
 *         setTheme: (theme) => set({ theme }),
 *       }),
 *       {
 *         name: 'my-store',
 *         storage: createJSONStorage(getPersistentStorage),
 *       }
 *     )
 *   )
 * );
 * ```
 */
export function createBoundStoreWithMiddleware<T>() {
  return (creator: any): CreateStoreResult<T> => {
    const store = createZustandStore<T>()(creator);

    const useStore: any = <U = T>(
      selector?: (state: T) => U,
      equalityFn?: (a: U, b: U) => boolean
    ) => {
      return useZustandStore(store, selector as any, equalityFn as any);
    };

    // Add getState to useStore hook
    useStore.getState = store.getState;

    return {
      store,
      useStore,
      getState: store.getState,
      setState: store.setState,
      subscribe: store.subscribe,
    };
  };
}
