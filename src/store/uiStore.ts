import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
// import { getPersistentStorage } from './storage';

export type Theme = 'dark' | 'light' | 'auto';

export interface PanelVisibility {
  sidebar: boolean;
  watchlist: boolean;
  orderBook: boolean;
  trades: boolean;
  chat: boolean;
  alerts: boolean;
}

export interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface UiStoreState {
  theme: Theme;
  panelVisibility: PanelVisibility;
  devConsoleVisible: boolean;
  sidebarCollapsed: boolean;
  commandPaletteOpen: boolean;
  notificationsEnabled: boolean;
  soundEnabled: boolean;
  animationsEnabled: boolean;
  compactMode: boolean;
  isAppLoading: boolean;
  appLoadingMessage: string | null;
  toasts: ToastMessage[];
  devConsoleOpen: boolean;

  // Actions
  setTheme: (theme: Theme) => void;
  setPanelVisibility: (panel: keyof PanelVisibility, visible: boolean) => void;
  togglePanel: (panel: keyof PanelVisibility) => void;
  setDevConsoleVisible: (visible: boolean) => void;
  toggleDevConsole: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  setSoundEnabled: (enabled: boolean) => void;
  setAnimationsEnabled: (enabled: boolean) => void;
  setCompactMode: (compact: boolean) => void;
  setLoading: (isLoading: boolean, message?: string | null) => void;
  addToast: (toast: Omit<ToastMessage, 'id'>) => void;
  removeToast: (id: string) => void;
  clearToasts: () => void;
  setDevConsoleOpen: (open: boolean) => void;
  closeDevtools: () => void;
  openDevtools: () => void;
  reset: () => void;
}

const defaultPanelVisibility: PanelVisibility = {
  sidebar: true,
  watchlist: true,
  orderBook: true,
  trades: true,
  chat: false,
  alerts: true,
};

const initialState = {
  theme: 'dark' as Theme,
  panelVisibility: defaultPanelVisibility,
  devConsoleVisible: false,
  sidebarCollapsed: false,
  commandPaletteOpen: false,
  notificationsEnabled: true,
  soundEnabled: true,
  animationsEnabled: true,
  compactMode: false,
  isAppLoading: false,
  appLoadingMessage: null as string | null,
  toasts: [] as ToastMessage[],
  devConsoleOpen: false,
};

// No-op storage to prevent synchronous I/O on import
const noOpStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
  clear: () => {},
} as Storage;

const storeResult = createBoundStoreWithMiddleware<UiStoreState>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        ...initialState,

        setTheme: theme => {
          if (get().theme === theme) return;
          set({ theme });
        },

        setPanelVisibility: (panel, visible) => {
          set(state => ({
            panelVisibility: {
              ...state.panelVisibility,
              [panel]: visible,
            },
          }));
        },

        togglePanel: panel => {
          set(state => ({
            panelVisibility: {
              ...state.panelVisibility,
              [panel]: !state.panelVisibility[panel],
            },
          }));
        },

        setDevConsoleVisible: visible => {
          if (get().devConsoleVisible === visible) return;
          set({ devConsoleVisible: visible });
        },

        toggleDevConsole: () => {
          set(state => ({ devConsoleVisible: !state.devConsoleVisible }));
        },

        setSidebarCollapsed: collapsed => {
          if (get().sidebarCollapsed === collapsed) return;
          set({ sidebarCollapsed: collapsed });
        },

        toggleSidebar: () => {
          set(state => ({ sidebarCollapsed: !state.sidebarCollapsed }));
        },

        setCommandPaletteOpen: open => {
          if (get().commandPaletteOpen === open) return;
          set({ commandPaletteOpen: open });
        },

        setNotificationsEnabled: enabled => {
          if (get().notificationsEnabled === enabled) return;
          set({ notificationsEnabled: enabled });
        },

        setSoundEnabled: enabled => {
          if (get().soundEnabled === enabled) return;
          set({ soundEnabled: enabled });
        },

        setAnimationsEnabled: enabled => {
          if (get().animationsEnabled === enabled) return;
          set({ animationsEnabled: enabled });
        },

        setCompactMode: compact => {
          if (get().compactMode === compact) return;
          set({ compactMode: compact });
        },

        setLoading: (isLoading, message = null) => {
          set({
            isAppLoading: isLoading,
            appLoadingMessage: message,
          });
        },

        addToast: toast => {
          const id = `toast-${Date.now()}-${Math.random()}`;
          const newToast: ToastMessage = {
            ...toast,
            id,
          };
          set(state => ({
            toasts: [...state.toasts, newToast],
          }));

          if (toast.duration) {
            setTimeout(() => {
              set(state => ({
                toasts: state.toasts.filter(t => t.id !== id),
              }));
            }, toast.duration);
          }
        },

        removeToast: id => {
          set(state => ({
            toasts: state.toasts.filter(t => t.id !== id),
          }));
        },

        clearToasts: () => {
          set({ toasts: [] });
        },

        setDevConsoleOpen: open => {
          if (get().devConsoleOpen === open) return;
          set({ devConsoleOpen: open });
        },

        closeDevtools: () => {
          set({ devConsoleOpen: false });
        },

        openDevtools: () => {
          set({ devConsoleOpen: true });
        },

        reset: () => {
          set(initialState);
        },
      }),
      {
        name: 'eclipse-ui-store',
        storage: createJSONStorage(() => noOpStorage),
        partialize: state => ({
          theme: state.theme,
          panelVisibility: state.panelVisibility,
          sidebarCollapsed: state.sidebarCollapsed,
          notificationsEnabled: state.notificationsEnabled,
          soundEnabled: state.soundEnabled,
          animationsEnabled: state.animationsEnabled,
          compactMode: state.compactMode,
          devConsoleVisible: state.devConsoleVisible,
        }),
      }
    )
  )
);

export const useUiStore = storeResult.useStore;
export const uiStore = storeResult.store;

export const useUIStore = useUiStore;

export const usePanelVisibility = (panel: keyof PanelVisibility) => {
  return useUiStore(state => state.panelVisibility[panel]);
};

export const useDevConsole = () => {
  return useUiStore(state => ({
    visible: state.devConsoleVisible,
    toggle: state.toggleDevConsole,
  }));
};

export const useTheme = () => {
  return useUiStore(state => state.theme);
};

export const useToasts = () => {
  return useUiStore(state => state.toasts);
};
