const createMemoryStorage = (): Storage => {
  let store: Record<string, string> = {};

  return {
    getItem: key => (key in store ? store[key] : null),
    setItem: (key, value) => {
      store[key] = value;
    },
    removeItem: key => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
    key: index => Object.keys(store)[index] ?? null,
    get length() {
      return Object.keys(store).length;
    },
  } as Storage;
};

// Lazy initialization - only check localStorage once when first accessed
let cachedStorage: Storage | null = null;

export const getPersistentStorage = (): Storage => {
  // Return cached result if already determined
  if (cachedStorage !== null) {
    return cachedStorage;
  }

  if (typeof window !== 'undefined') {
    try {
      // Test localStorage access with minimal operations
      const testKey = '__storage_test__';
      window.localStorage.setItem(testKey, '1');
      window.localStorage.removeItem(testKey);
      cachedStorage = window.localStorage;
      return cachedStorage;
    } catch (e) {
      // localStorage is blocked by Tauri security, use in-memory fallback
      console.warn('[Storage] localStorage blocked, using in-memory storage', e);
      cachedStorage = createMemoryStorage();
      return cachedStorage;
    }
  }

  cachedStorage = createMemoryStorage();
  return cachedStorage;
};
