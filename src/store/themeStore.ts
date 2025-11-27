import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
// import { getPersistentStorage } from './storage';

export interface ThemeColors {
  background: string;
  backgroundSecondary: string;
  backgroundTertiary: string;
  text: string;
  textSecondary: string;
  textMuted: string;
  primary: string;
  primaryHover: string;
  primaryActive: string;
  accent: string;
  accentHover: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  border: string;
  borderHover: string;
  chartBullish: string;
  chartBearish: string;
  chartNeutral: string;
  gradientStart: string;
  gradientMiddle: string;
  gradientEnd: string;
}

export interface ThemeDefinition {
  id: string;
  name: string;
  type: 'preset' | 'custom';
  colors: ThemeColors;
}

export interface ThemeStoreState {
  activeThemeId: string;
  currentTheme: ThemeDefinition;
  customThemes: ThemeDefinition[];
  setActiveTheme: (id: string) => void;
  createCustomTheme: (name: string, colors: ThemeColors) => ThemeDefinition;
  updateCustomTheme: (id: string, colors: Partial<ThemeColors>) => void;
  deleteCustomTheme: (id: string) => void;
  exportTheme: (id: string) => string;
  importTheme: (payload: string) => ThemeDefinition | null;
  listThemes: () => ThemeDefinition[];
}

const createCustomThemeId = () =>
  `custom-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const BUILTIN_THEMES: ThemeDefinition[] = [
  {
    id: 'eclipse',
    name: 'Eclipse',
    type: 'preset',
    colors: {
      background: '#07090f',
      backgroundSecondary: '#0f1424',
      backgroundTertiary: '#161d33',
      text: '#f4f5ff',
      textSecondary: '#e0e4ff',
      textMuted: '#9ca3c7',
      primary: '#9f7bff',
      primaryHover: '#b08bff',
      primaryActive: '#8a67f5',
      accent: '#ffb347',
      accentHover: '#ff9234',
      success: '#22c55e',
      warning: '#f97316',
      error: '#ef4444',
      info: '#38bdf8',
      border: '#1e2439',
      borderHover: '#2a314e',
      chartBullish: '#22c55e',
      chartBearish: '#ef4444',
      chartNeutral: '#94a3b8',
      gradientStart: '#050817',
      gradientMiddle: '#111a33',
      gradientEnd: '#1a2849',
    },
  },
  {
    id: 'midnight',
    name: 'Midnight',
    type: 'preset',
    colors: {
      background: '#05070c',
      backgroundSecondary: '#0b101c',
      backgroundTertiary: '#10182a',
      text: '#f1f5ff',
      textSecondary: '#dbe4ff',
      textMuted: '#98a2c3',
      primary: '#4f9bff',
      primaryHover: '#6fb1ff',
      primaryActive: '#3f84e5',
      accent: '#22d3ee',
      accentHover: '#06b6d4',
      success: '#34d399',
      warning: '#fbbf24',
      error: '#fb7185',
      info: '#67e8f9',
      border: '#1a2337',
      borderHover: '#24304c',
      chartBullish: '#34d399',
      chartBearish: '#fb7185',
      chartNeutral: '#cbd5f5',
      gradientStart: '#050812',
      gradientMiddle: '#0e1425',
      gradientEnd: '#12203c',
    },
  },
  {
    id: 'cyber',
    name: 'Cyber',
    type: 'preset',
    colors: {
      background: '#09020f',
      backgroundSecondary: '#14051b',
      backgroundTertiary: '#1d0a29',
      text: '#fdf4ff',
      textSecondary: '#f5d0fe',
      textMuted: '#f0abfc',
      primary: '#f472b6',
      primaryHover: '#ec4899',
      primaryActive: '#db2777',
      accent: '#10b981',
      accentHover: '#059669',
      success: '#34d399',
      warning: '#fde047',
      error: '#fb7185',
      info: '#38bdf8',
      border: '#2b103c',
      borderHover: '#3f1654',
      chartBullish: '#10b981',
      chartBearish: '#fb7185',
      chartNeutral: '#f0abfc',
      gradientStart: '#120323',
      gradientMiddle: '#22063d',
      gradientEnd: '#300956',
    },
  },
  {
    id: 'lunar',
    name: 'Lunar',
    type: 'preset',
    colors: {
      background: '#0a0c12',
      backgroundSecondary: '#10131f',
      backgroundTertiary: '#171c2d',
      text: '#f8fafc',
      textSecondary: '#e2e8f0',
      textMuted: '#cbd5f5',
      primary: '#60a5fa',
      primaryHover: '#3b82f6',
      primaryActive: '#2563eb',
      accent: '#facc15',
      accentHover: '#eab308',
      success: '#86efac',
      warning: '#fde047',
      error: '#f87171',
      info: '#7dd3fc',
      border: '#1f2436',
      borderHover: '#2c3351',
      chartBullish: '#86efac',
      chartBearish: '#f87171',
      chartNeutral: '#bfdbfe',
      gradientStart: '#090c16',
      gradientMiddle: '#13182b',
      gradientEnd: '#1b2340',
    },
  },
];

const findThemeById = (id: string, customThemes: ThemeDefinition[]) => {
  return BUILTIN_THEMES.concat(customThemes).find(theme => theme.id === id);
};

// No-op storage to prevent synchronous I/O on import
const noOpStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
  clear: () => {},
} as Storage;

const storeResult = createBoundStoreWithMiddleware<ThemeStoreState>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        activeThemeId: BUILTIN_THEMES[0].id,
        currentTheme: BUILTIN_THEMES[0],
        customThemes: [],
        setActiveTheme: id =>
          set(state => {
            const nextTheme = findThemeById(id, state.customThemes) ?? BUILTIN_THEMES[0];
            return {
              activeThemeId: nextTheme.id,
              currentTheme: nextTheme,
            };
          }),
        createCustomTheme: (name, colors) => {
          const theme: ThemeDefinition = {
            id: createCustomThemeId(),
            name,
            type: 'custom',
            colors: { ...colors },
          };

          set(state => ({
            customThemes: [...state.customThemes, theme],
            activeThemeId: theme.id,
            currentTheme: theme,
          }));

          return theme;
        },
        updateCustomTheme: (id, colors) =>
          set(state => ({
            customThemes: state.customThemes.map(theme =>
              theme.id === id ? { ...theme, colors: { ...theme.colors, ...colors } } : theme
            ),
            currentTheme:
              state.currentTheme.id === id
                ? { ...state.currentTheme, colors: { ...state.currentTheme.colors, ...colors } }
                : state.currentTheme,
          })),
        deleteCustomTheme: id =>
          set(state => {
            const nextCustomThemes = state.customThemes.filter(theme => theme.id !== id);
            const isActiveTheme = state.activeThemeId === id;
            if (!isActiveTheme) {
              return { customThemes: nextCustomThemes };
            }

            const fallbackTheme = nextCustomThemes[0] ?? BUILTIN_THEMES[0];
            return {
              customThemes: nextCustomThemes,
              activeThemeId: fallbackTheme.id,
              currentTheme: fallbackTheme,
            };
          }),
        exportTheme: id => {
          const theme = findThemeById(id, get().customThemes) ?? get().currentTheme;
          return JSON.stringify({ id: theme.id, name: theme.name, colors: theme.colors }, null, 2);
        },
        importTheme: payload => {
          try {
            const parsed = JSON.parse(payload);
            if (!parsed?.colors) {
              return null;
            }

            const id: string =
              parsed.id && typeof parsed.id === 'string'
                ? String(parsed.id)
                : createCustomThemeId();
            const theme: ThemeDefinition = {
              id: id.startsWith('custom-') ? id : createCustomThemeId(),
              name: parsed.name || 'Imported Theme',
              type: 'custom',
              colors: { ...parsed.colors },
            };

            set(state => ({
              customThemes: [...state.customThemes, theme],
              activeThemeId: theme.id,
              currentTheme: theme,
            }));

            return theme;
          } catch (error) {
            console.error('[themeStore] Failed to import theme', error);
            return null;
          }
        },
        listThemes: () => BUILTIN_THEMES.concat(get().customThemes),
      }),
      {
        name: 'eclipse-theme-store',
        storage: createJSONStorage(() => noOpStorage),
        partialize: state => ({
          activeThemeId: state.activeThemeId,
          customThemes: state.customThemes,
        }),
        onRehydrateStorage: () => state => {
          if (!state) {
            return;
          }

          const theme = findThemeById(state.activeThemeId, state.customThemes) ?? BUILTIN_THEMES[0];
          state.currentTheme = theme;
          state.activeThemeId = theme.id;
        },
      }
    )
  )
);

export const useThemeStore = storeResult.useStore;
export const themeStore = storeResult.store;

export const useCurrentTheme = () => {
  return useThemeStore(state => state.currentTheme);
};

export const useCustomThemes = () => {
  return useThemeStore(state => state.customThemes);
};
