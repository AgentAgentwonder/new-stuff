import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
// import { getPersistentStorage } from './storage';

const DEFAULTS = {
  fontScale: 1,
  highContrastMode: false,
  reducedMotion: false,
  screenReaderOptimizations: false,
  keyboardNavigationHints: false,
  focusIndicatorEnhanced: false,
} as const;

// No-op storage to prevent synchronous I/O on import
const noOpStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
  clear: () => {},
} as Storage;

export interface AccessibilityState {
  fontScale: number;
  highContrastMode: boolean;
  reducedMotion: boolean;
  screenReaderOptimizations: boolean;
  keyboardNavigationHints: boolean;
  focusIndicatorEnhanced: boolean;
  setFontScale: (value: number) => void;
  toggleHighContrast: () => void;
  toggleReducedMotion: () => void;
  toggleScreenReaderOptimizations: () => void;
  toggleKeyboardNavigationHints: () => void;
  toggleFocusIndicatorEnhanced: () => void;
  resetToDefaults: () => void;
}

const clampFontScale = (value: number) => {
  if (!Number.isFinite(value)) {
    return DEFAULTS.fontScale;
  }

  return Math.min(2, Math.max(1, value));
};

const storeResult = createBoundStoreWithMiddleware<AccessibilityState>()(
  subscribeWithSelector(
    persist(
      set => ({
        ...DEFAULTS,
        setFontScale: value => set(() => ({ fontScale: clampFontScale(value) })),
        toggleHighContrast: () => set(state => ({ highContrastMode: !state.highContrastMode })),
        toggleReducedMotion: () => set(state => ({ reducedMotion: !state.reducedMotion })),
        toggleScreenReaderOptimizations: () =>
          set(state => ({ screenReaderOptimizations: !state.screenReaderOptimizations })),
        toggleKeyboardNavigationHints: () =>
          set(state => ({ keyboardNavigationHints: !state.keyboardNavigationHints })),
        toggleFocusIndicatorEnhanced: () =>
          set(state => ({ focusIndicatorEnhanced: !state.focusIndicatorEnhanced })),
        resetToDefaults: () => set(() => ({ ...DEFAULTS })),
      }),
      {
        name: 'eclipse-accessibility-store',
        storage: createJSONStorage(() => noOpStorage),
      }
    )
  )
);

export const useAccessibilityStore = storeResult.useStore;
export const accessibilityStore = storeResult.store;

export const useFontScale = () => {
  return useAccessibilityStore(state => state.fontScale);
};

export const useHighContrastMode = () => {
  return useAccessibilityStore(state => state.highContrastMode);
};

export const useReducedMotion = () => {
  return useAccessibilityStore(state => state.reducedMotion);
};
