import type { Position, PortfolioAnalytics, SectorAllocation, ConcentrationAlert } from '../types';
import { createBoundStore } from './createBoundStore';

interface AnalyticsCache {
  data: PortfolioAnalytics;
  timestamp: string;
  ttl: number;
}

interface PortfolioStoreState {
  positions: Position[];
  analyticsCache: Record<string, AnalyticsCache>;
  sectorAllocations: SectorAllocation[];
  concentrationAlerts: ConcentrationAlert[];
  totalValue: number;
  totalPnl: number;
  totalPnlPercent: number;
  isLoading: boolean;
  error: string | null;

  // Actions
  setPositions: (positions: Position[]) => void;
  fetchAnalytics: (walletAddress: string, forceRefresh?: boolean) => Promise<PortfolioAnalytics>;
  fetchSectorAllocations: (walletAddress: string) => Promise<void>;
  checkConcentrationAlerts: () => void;
  calculateTotals: () => void;
  refreshPortfolio: (walletAddress: string) => Promise<void>;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  positions: [],
  analyticsCache: {},
  sectorAllocations: [],
  concentrationAlerts: [],
  totalValue: 0,
  totalPnl: 0,
  totalPnlPercent: 0,
  isLoading: false,
  error: null,
};

const CACHE_TTL_MS = 5 * 60 * 1000; // 5 minutes

const storeResult = createBoundStore<PortfolioStoreState>((set, get) => ({
  ...initialState,

  setPositions: positions => {
    set({ positions });
    get().calculateTotals();
    get().checkConcentrationAlerts();
  },

  fetchAnalytics: async (walletAddress: string, forceRefresh = false) => {
    const cached = get().analyticsCache[walletAddress];
    const now = Date.now();

    if (!forceRefresh && cached && now - new Date(cached.timestamp).getTime() < cached.ttl) {
      return cached.data;
    }

    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const analytics = await invoke<PortfolioAnalytics>('portfolio_get_analytics', {
        walletAddress,
      });

      set(state => ({
        analyticsCache: {
          ...state.analyticsCache,
          [walletAddress]: {
            data: analytics,
            timestamp: new Date().toISOString(),
            ttl: CACHE_TTL_MS,
          },
        },
        isLoading: false,
      }));

      return analytics;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  fetchSectorAllocations: async (walletAddress: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const allocations = await invoke<SectorAllocation[]>('portfolio_get_sector_allocation', {
        walletAddress,
      });
      set({ sectorAllocations: allocations, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  checkConcentrationAlerts: () => {
    const { positions } = get();
    const alerts: ConcentrationAlert[] = [];

    positions.forEach(pos => {
      if (pos.allocation >= 40) {
        alerts.push({
          id: `alert-${pos.symbol}-${Date.now()}`,
          symbol: pos.symbol,
          allocation: pos.allocation,
          severity: 'critical',
          message: `${pos.symbol} represents ${pos.allocation.toFixed(1)}% of your portfolio. Critical concentration risk detected.`,
          threshold: 40,
          createdAt: new Date().toISOString(),
        });
      } else if (pos.allocation >= 30) {
        alerts.push({
          id: `alert-${pos.symbol}-${Date.now()}`,
          symbol: pos.symbol,
          allocation: pos.allocation,
          severity: 'warning',
          message: `${pos.symbol} represents ${pos.allocation.toFixed(1)}% of your portfolio. Consider diversifying.`,
          threshold: 30,
          createdAt: new Date().toISOString(),
        });
      }
    });

    set({ concentrationAlerts: alerts });
  },

  calculateTotals: () => {
    const { positions } = get();
    const totalValue = positions.reduce((sum, pos) => sum + pos.value, 0);
    const totalPnl = positions.reduce((sum, pos) => sum + (pos.pnl || 0), 0);
    const totalPnlPercent = totalValue > 0 ? (totalPnl / (totalValue - totalPnl)) * 100 : 0;

    set({ totalValue, totalPnl, totalPnlPercent });
  },

  refreshPortfolio: async (walletAddress: string) => {
    set({ isLoading: true, error: null });
    try {
      await Promise.all([
        get().fetchAnalytics(walletAddress, true),
        get().fetchSectorAllocations(walletAddress),
      ]);
      set({ isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setError: error => {
    if (get().error === error) return;
    set({ error });
  },

  reset: () => {
    set(initialState);
  },
}));

export const usePortfolioStore = storeResult.useStore;
export const portfolioStore = storeResult.store;

export const usePositions = () => {
  return usePortfolioStore(state => state.positions);
};

export const useSectorAllocations = () => {
  return usePortfolioStore(state => state.sectorAllocations);
};

export const useConcentrationAlerts = () => {
  return usePortfolioStore(state => state.concentrationAlerts);
};

export const usePortfolioTotals = () => {
  return usePortfolioStore(state => ({
    totalValue: state.totalValue,
    totalPnl: state.totalPnl,
    totalPnlPercent: state.totalPnlPercent,
  }));
};

export const usePortfolioStatus = () => {
  return usePortfolioStore(state => ({
    isLoading: state.isLoading,
    error: state.error,
  }));
};

export const useAnalyticsCache = () => {
  return usePortfolioStore(state => state.analyticsCache);
};
