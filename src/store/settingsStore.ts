import { useCallback, useMemo } from 'react';
import { persist, createJSONStorage } from 'zustand/middleware';
import { subscribeWithSelector } from 'zustand/middleware';
import { createBoundStoreWithMiddleware } from './createBoundStore';
import { useShallow } from '@/store/createBoundStore';
import { getPersistentStorage } from './storage';
import { errorLogger } from '@/utils/errorLogger';

export interface SettingsState {
  databaseUrl: string;
  sentrySdn: string;
  claudeApiKey: string;
  openaiApiKey: string;
  llmProvider: 'claude' | 'gpt4';
  twitterBearerToken: string;
  paperTradingEnabled: boolean;
  paperTradingBalance: number;
  selectedCrypto: string;
  buyInAmounts: number[];
  defaultBuyInAmount: number;
  minMarketCap: number;
  theme: 'eclipse' | 'midnight' | 'cyber' | 'lunar';
  phantomConnected: boolean;
  phantomAddress: string;
  updateSetting: <
    K extends keyof Omit<
      SettingsState,
      | 'updateSetting'
      | 'togglePaperTrading'
      | 'addBuyInPreset'
      | 'removeBuyInPreset'
      | 'connectPhantom'
      | 'disconnectPhantom'
      | 'resetSettings'
    >,
  >(
    key: K,
    value: SettingsState[K]
  ) => void;
  togglePaperTrading: () => void;
  addBuyInPreset: (amount: number) => void;
  removeBuyInPreset: (amount: number) => void;
  connectPhantom: (address: string) => void;
  disconnectPhantom: () => void;
  resetSettings: () => void;
}

const DEFAULTS: Omit<
  SettingsState,
  | 'updateSetting'
  | 'togglePaperTrading'
  | 'addBuyInPreset'
  | 'removeBuyInPreset'
  | 'connectPhantom'
  | 'disconnectPhantom'
  | 'resetSettings'
> = {
  databaseUrl: '',
  sentrySdn: '',
  claudeApiKey: '',
  openaiApiKey: '',
  llmProvider: 'claude',
  twitterBearerToken: '',
  paperTradingEnabled: false,
  paperTradingBalance: 10000,
  selectedCrypto: 'SOL',
  buyInAmounts: [10, 25, 50, 100],
  defaultBuyInAmount: 50,
  minMarketCap: 25000000,
  theme: 'eclipse',
  phantomConnected: false,
  phantomAddress: '',
};

// No-op storage to prevent synchronous I/O on import
// Hydration will be handled separately
const noOpStorage = {
  getItem: () => null,
  setItem: () => {},
  removeItem: () => {},
  clear: () => {},
} as Storage;

const storeResult = createBoundStoreWithMiddleware<SettingsState>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        ...DEFAULTS,
        updateSetting: (key, value) => {
          set({ [key]: value });
          errorLogger.info(`Settings: Updated ${key}`, 'settingsStore');
        },
        togglePaperTrading: () =>
          set(state => {
            const newValue = !state.paperTradingEnabled;
            errorLogger.info(
              `Settings: Paper trading ${newValue ? 'enabled' : 'disabled'}`,
              'settingsStore'
            );
            return { paperTradingEnabled: newValue };
          }),
        addBuyInPreset: amount => {
          const current = get().buyInAmounts;
          if (current.includes(amount)) {
            errorLogger.info(`Settings: Buy-in preset $${amount} already exists`, 'settingsStore');
            return;
          }
          const updated = [...current, amount].sort((a, b) => a - b);
          set({ buyInAmounts: updated });
          errorLogger.info(`Settings: Added buy-in preset $${amount}`, 'settingsStore');
        },
        removeBuyInPreset: amount => {
          const state = get();
          const updated = state.buyInAmounts.filter(a => a !== amount);
          const updates: Partial<SettingsState> = { buyInAmounts: updated };
          if (state.defaultBuyInAmount === amount && updated.length > 0) {
            updates.defaultBuyInAmount = updated[0];
            errorLogger.info(
              `Settings: Removed buy-in preset $${amount} and reset default to $${updated[0]}`,
              'settingsStore'
            );
          } else {
            errorLogger.info(`Settings: Removed buy-in preset $${amount}`, 'settingsStore');
          }
          set(updates);
        },
        connectPhantom: address => {
          set({ phantomConnected: true, phantomAddress: address });
          errorLogger.info(`Settings: Connected Phantom wallet ${address}`, 'settingsStore');
        },
        disconnectPhantom: () => {
          set({ phantomConnected: false, phantomAddress: '' });
          errorLogger.info('Settings: Disconnected Phantom wallet', 'settingsStore');
        },
        resetSettings: () => {
          set(DEFAULTS);
          errorLogger.info('Settings: Reset all settings to defaults', 'settingsStore');
        },
      }),
      {
        name: 'eclipse-settings-store',
        storage: createJSONStorage(() => noOpStorage),
        version: 1,
        migrate: (persistedState: any, version: number) => {
          // Note: Real migration requires actual storage access
          // This is deferred during hydration phase
          return persistedState as SettingsState;
        },
      }
    )
  )
);

// CRITICAL EXPORTS
export const useSettingsStore = storeResult.useStore;
export const settingsStore = storeResult.store;

// Selector hooks with memoization and shallow comparison
export const usePaperTrading = () => {
  // Use primitive selectors instead of object selector to avoid comparison issues
  const enabled = useSettingsStore(state => state.paperTradingEnabled);
  const balance = useSettingsStore(state => state.paperTradingBalance);
  
  // Return memoized object - only recreates if enabled or balance changes
  return useMemo(() => ({
    enabled,
    balance,
  }), [enabled, balance]);
};

export const useQuickBuys = () => {
  const amounts = useSettingsStore(state => state.buyInAmounts);
  const defaultAmount = useSettingsStore(state => state.defaultBuyInAmount);
  
  return useMemo(() => ({
    amounts,
    defaultAmount,
  }), [amounts, defaultAmount]);
};

export const useSelectedCrypto = () => {
  const selector = useCallback((state: SettingsState) => state.selectedCrypto, []);
  return useSettingsStore(selector, useShallow);
};

export const useMinMarketCap = () => {
  const selector = useCallback((state: SettingsState) => state.minMarketCap, []);
  return useSettingsStore(selector, useShallow);
};

export const useLLMProvider = () => {
  const provider = useSettingsStore(state => state.llmProvider);
  const claudeKey = useSettingsStore(state => state.claudeApiKey);
  const openaiKey = useSettingsStore(state => state.openaiApiKey);
  
  return useMemo(() => ({
    provider,
    claudeKey,
    openaiKey,
  }), [provider, claudeKey, openaiKey]);
};

export const usePhantomWallet = () => {
  const connected = useSettingsStore(state => state.phantomConnected);
  const address = useSettingsStore(state => state.phantomAddress);
  
  return useMemo(() => ({
    connected,
    address,
  }), [connected, address]);
};
