import { create } from 'zustand';
import { Memecoin, TradeSignal, TradeExecution, RiskConfig, AITradingConfig, WalletState, PortfolioState } from '../types/trading';

interface TradingStore {
  // Market Data
  coins: Map<string, Memecoin>;
  watchedCoins: string[];
  priceHistory: Map<string, { price: number; timestamp: number }[]>;
  
  // Trading State
  signals: TradeSignal[];
  pendingTrades: TradeExecution[];
  tradeHistory: TradeExecution[];
  
  // Configuration
  riskConfig: RiskConfig;
  aiConfig: AITradingConfig;
  
  // Wallet & Portfolio
  wallet: WalletState;
  portfolio: PortfolioState;
  
  // Actions
  updateCoin: (coin: Memecoin) => void;
  updatePrice: (address: string, price: number) => void;
  addSignal: (signal: TradeSignal) => void;
  addTrade: (trade: TradeExecution) => void;
  updateTradeStatus: (id: string, status: TradeExecution['status'], signature?: string, error?: string) => void;
  setRiskConfig: (config: RiskConfig) => void;
  setAIConfig: (config: AITradingConfig) => void;
  setWallet: (wallet: WalletState) => void;
  updatePortfolio: (portfolio: PortfolioState) => void;
  watchCoin: (address: string) => void;
  unwatchCoin: (address: string) => void;
}

const defaultRiskConfig: RiskConfig = {
  maxPositionSize: 1000,
  maxSlippage: 2.5,
  minLiquidity: 10000,
  minHolderCount: 100,
  maxSingleTrade: 500,
  stopLossPercentage: 15,
  takeProfitPercentage: 50,
  greenThreshold: 75,
  yellowThreshold: 50,
  notifyYellow: true,
};

const defaultAIConfig: AITradingConfig = {
  enabled: false,
  autoTradeGreen: false,
  notifyYellow: true,
  maxDailyTrades: 10,
  model: 'local',
  trainingData: [],
};

export const useTradingStore = create<TradingStore>((set) => ({
  coins: new Map(),
  watchedCoins: [],
  priceHistory: new Map(),
  signals: [],
  pendingTrades: [],
  tradeHistory: [],
  riskConfig: defaultRiskConfig,
  aiConfig: defaultAIConfig,
  wallet: {
    connected: false,
    address: null,
    balance: 0,
    tokenBalances: [],
  },
  portfolio: {
    totalValue: 0,
    positions: [],
    pnl24h: 0,
    pnlTotal: 0,
  },

  updateCoin: (coin) => {
    set((state) => {
      const coins = new Map(state.coins);
      const existing = coins.get(coin.address);
      coins.set(coin.address, { ...existing, ...coin, lastUpdated: Date.now() });
      return { coins };
    });
  },

  updatePrice: (address, price) => {
    set((state) => {
      const priceHistory = new Map(state.priceHistory);
      const history = priceHistory.get(address) || [];
      history.push({ price, timestamp: Date.now() });
      // Keep last 1000 price points
      if (history.length > 1000) history.shift();
      priceHistory.set(address, history);
      return { priceHistory };
    });
  },

  addSignal: (signal) => {
    set((state) => ({
      signals: [signal, ...state.signals].slice(0, 100), // Keep last 100 signals
    }));
  },

  addTrade: (trade) => {
    set((state) => ({
      pendingTrades: [...state.pendingTrades, trade],
    }));
  },

  updateTradeStatus: (id, status, signature, error) => {
    set((state) => {
      const pendingTrades = state.pendingTrades.filter((t) => t.id !== id);
      const trade = state.pendingTrades.find((t) => t.id === id);
      if (trade) {
        const updated = { ...trade, status, signature, error, executedAt: Date.now() };
        return {
          pendingTrades,
          tradeHistory: [updated, ...state.tradeHistory].slice(0, 1000),
        };
      }
      return { pendingTrades };
    });
  },

  setRiskConfig: (config) => set({ riskConfig: config }),
  setAIConfig: (config) => set({ aiConfig: config }),
  setWallet: (wallet) => set({ wallet }),
  updatePortfolio: (portfolio) => set({ portfolio }),
  
  watchCoin: (address) => {
    set((state) => ({
      watchedCoins: [...new Set([...state.watchedCoins, address])],
    }));
  },
  
  unwatchCoin: (_address) => {
    set((state) => ({
      watchedCoins: state.watchedCoins.filter((a) => a !== _address),
    }));
  },
}));
