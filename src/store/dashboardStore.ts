import { create } from 'zustand';
import { MemecoinToken, heliusService } from '../services/heliusService';
import { jupiterService, type SwapResult } from '../services/jupiterService';
import { marketDataService, type CandleData } from '../services/marketDataService';

export interface Trade {
  id: string;
  type: 'buy' | 'sell';
  token: string;
  amount: number;
  price: number;
  total: number;
  time: string;
  status: 'pending' | 'filled' | 'cancelled' | 'failed';
  txSignature?: string;
}

export interface Holding {
  id: string;
  symbol: string;
  name: string;
  amount: number;
  value: number;
  price: number;
  change24h: number;
  pnl: number;
}

export interface BotSettings {
  isRunning: boolean;
  isSimulation: boolean;
  minProfitThreshold: number;
  maxSlippage: number;
  autoCompound: boolean;
  riskLevel: 'low' | 'medium' | 'high';
  tradeAmount: number;
  maxPositionSize: number;
  stopLoss: number;
  takeProfit: number;
}

export interface WalletInfo {
  address: string;
  connected: boolean;
  balance: number;
  tokens: HeliusToken[];
}

interface HeliusToken {
  mint: string;
  amount: string;
  decimals: number;
  uiAmount: number;
  symbol?: string;
  name?: string;
}

interface DashboardState {
  // Market data
  solPrice: number;
  priceChange: number;
  priceDirection: 'up' | 'down';
  coins: MemecoinToken[];
  selectedCoin: MemecoinToken | null;
  candleData: CandleData[];
  
  // Trading
  trades: Trade[];
  balance: number;
  holdings: Holding[];
  
  // Bot
  botSettings: BotSettings;
  botStats: {
    totalTrades: number;
    profitableTrades: number;
    totalPnl: number;
    winRate: number;
  };
  
  // Wallet
  wallet: WalletInfo;
  
  // UI State
  activeTab: 'dashboard' | 'portfolio' | 'settings' | 'history';
  isLoading: boolean;
  error: string | null;

  // Actions
  initialize: () => Promise<void>;
  setActiveTab: (tab: 'dashboard' | 'portfolio' | 'settings' | 'history') => void;
  selectCoin: (coin: MemecoinToken) => void;
  
  // Trading actions
  executeTrade: (type: 'buy' | 'sell', token: string, amount: number) => Promise<void>;
  
  // Bot actions
  toggleBot: () => void;
  toggleSimulation: () => void;
  updateBotSettings: (settings: Partial<BotSettings>) => void;
  
  // Wallet actions
  connectWallet: (address: string) => Promise<void>;
  disconnectWallet: () => void;
  refreshBalances: () => Promise<void>;
  
  // Data refresh
  refreshMarketData: () => Promise<void>;
}

const initialBotSettings: BotSettings = {
  isRunning: false,
  isSimulation: true,
  minProfitThreshold: 1.0,
  maxSlippage: 5.0,
  autoCompound: false,
  riskLevel: 'medium',
  tradeAmount: 0.05,
  maxPositionSize: 0.5,
  stopLoss: 10,
  takeProfit: 50,
};

const initialWallet: WalletInfo = {
  address: '',
  connected: false,
  balance: 0.5,
  tokens: [],
};

const sampleTrades: Trade[] = [
  {
    id: '1',
    type: 'buy',
    token: 'DOGE2',
    amount: 0.05,
    price: 0.00012,
    total: 0.05,
    time: new Date(Date.now() - 3600000).toISOString(),
    status: 'filled',
  },
  {
    id: '2',
    type: 'sell',
    token: 'PUMP',
    amount: 0.03,
    price: 0.00045,
    total: 0.03,
    time: new Date(Date.now() - 7200000).toISOString(),
    status: 'filled',
  },
  {
    id: '3',
    type: 'buy',
    token: 'YOLO',
    amount: 0.1,
    price: 0.00008,
    total: 0.1,
    time: new Date(Date.now() - 10800000).toISOString(),
    status: 'filled',
  },
];

const sampleHoldings: Holding[] = [
  {
    id: '1',
    symbol: 'DOGE2',
    name: 'DOGE2 Token',
    amount: 1500,
    value: 0.18,
    price: 0.00012,
    change24h: 5.2,
    pnl: 0.02,
  },
  {
    id: '2',
    symbol: 'YOLO',
    name: 'YOLO Token',
    amount: 2500,
    value: 0.2,
    price: 0.00008,
    change24h: -2.3,
    pnl: -0.01,
  },
];

export const useDashboardStore = create<DashboardState>((set, get) => ({
  // Initial state
  solPrice: 148.65,
  priceChange: 3.21,
  priceDirection: 'up',
  coins: [],
  selectedCoin: null,
  candleData: [],
  trades: sampleTrades,
  balance: 0.5,
  holdings: sampleHoldings,
  botSettings: initialBotSettings,
  botStats: {
    totalTrades: 0,
    profitableTrades: 0,
    totalPnl: 0,
    winRate: 0,
  },
  wallet: initialWallet,
  activeTab: 'dashboard',
  isLoading: false,
  error: null,

  initialize: async () => {
    set({ isLoading: true, error: null });
    try {
      // Get initial market data
      const marketData = marketDataService.getMarketData();
      
      set({
        solPrice: marketData.solPrice,
        coins: marketData.tokens,
        isLoading: false,
      });

      // Subscribe to market data updates
      marketDataService.subscribe((data) => {
        set({
          solPrice: data.solPrice,
          coins: data.tokens,
          priceChange: (Math.random() - 0.5) * 10,
          priceDirection: Math.random() > 0.5 ? 'up' : 'down',
        });
      });

      // Generate candle data for the first token if available
      if (marketData.tokens.length > 0) {
        const firstCoin = marketData.tokens[0];
        const candles = marketDataService.generateCandleData(firstCoin.price);
        set({ selectedCoin: firstCoin, candleData: candles });
      }
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  setActiveTab: (tab) => {
    set({ activeTab: tab });
  },

  selectCoin: (coin) => {
    const candles = marketDataService.generateCandleData(coin.price);
    set({ selectedCoin: coin, candleData: candles });
  },

  executeTrade: async (type, token, amount) => {
    const state = get();
    const price = state.selectedCoin?.price || 0.0001;
    const total = amount;

    // Check if we have enough balance
    if (type === 'buy' && amount > state.balance) {
      set({ error: 'Insufficient balance' });
      return;
    }

    try {
      // In simulation mode or if wallet not connected, use mock trade
      if (state.botSettings.isSimulation || !state.wallet.connected) {
        const newTrade: Trade = {
          id: Date.now().toString(),
          type,
          token,
          amount,
          price,
          total: amount * price,
          time: new Date().toISOString(),
          status: 'filled',
        };

        set({
          trades: [newTrade, ...state.trades],
          balance: type === 'buy' ? state.balance - amount : state.balance + amount,
        });

        // Update holdings
        if (type === 'buy') {
          const existingHolding = state.holdings.find(h => h.symbol === token);
          if (existingHolding) {
            set({
              holdings: state.holdings.map(h =>
                h.symbol === token
                  ? { ...h, amount: h.amount + amount / price, value: h.value + amount }
                  : h
              ),
            });
          } else {
            set({
              holdings: [
                ...state.holdings,
                {
                  id: Date.now().toString(),
                  symbol: token,
                  name: `${token} Token`,
                  amount: amount / price,
                  value: amount,
                  price,
                  change24h: 0,
                  pnl: 0,
                },
              ],
            });
          }
        } else {
          // Sell
          set({
            holdings: state.holdings.map(h =>
              h.symbol === token
                ? { ...h, amount: h.amount - amount / price, value: h.value - amount }
                : h
            ).filter(h => h.amount > 0),
          });
        }
      } else {
        // Real trade via Jupiter
        const result: SwapResult | null = await jupiterService.quickSwap(
          state.wallet.address,
          type === 'buy' ? 'So11111111111111111111111111111111111111112' : token, // SOL mint
          type === 'buy' ? token : 'So11111111111111111111111111111111111111112',
          amount * 1e9 // Convert to lamports
        );

        if (result) {
          const newTrade: Trade = {
            id: result.txid,
            type,
            token,
            amount,
            price,
            total: amount * price,
            time: new Date().toISOString(),
            status: 'filled',
            txSignature: result.txid,
          };

          set({
            trades: [newTrade, ...state.trades],
            balance: type === 'buy' ? state.balance - amount : state.balance + amount,
          });
        }
      }

      // Update bot stats
      const trades = get().trades;
      const profitableTrades = trades.filter(t => t.type === 'sell').length;
      set({
        botStats: {
          totalTrades: trades.length,
          profitableTrades,
          totalPnl: trades.reduce((acc, t) => acc + (t.type === 'sell' ? t.total * 0.1 : 0), 0),
          winRate: trades.length > 0 ? (profitableTrades / trades.length) * 100 : 0,
        },
      });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  toggleBot: () => {
    set(state => ({
      botSettings: { ...state.botSettings, isRunning: !state.botSettings.isRunning },
    }));
  },

  toggleSimulation: () => {
    set(state => ({
      botSettings: { ...state.botSettings, isSimulation: !state.botSettings.isSimulation },
    }));
  },

  updateBotSettings: (settings) => {
    set(state => ({
      botSettings: { ...state.botSettings, ...settings },
    }));
  },

  connectWallet: async (address) => {
    set({ isLoading: true, error: null });
    try {
      // Try to fetch real wallet data
      const tokens = await heliusService.getWalletBalances(address);
      const balance = tokens.find(t => t.mint === 'So11111111111111111111111111111111111111112')?.uiAmount || 0;

      set({
        wallet: {
          address,
          connected: true,
          balance,
          tokens,
        },
        isLoading: false,
      });
    } catch (error) {
      // Fallback to mock data if real connection fails
      set({
        wallet: {
          address,
          connected: true,
          balance: 0.5,
          tokens: [],
        },
        isLoading: false,
      });
    }
  },

  disconnectWallet: () => {
    set({
      wallet: initialWallet,
    });
  },

  refreshBalances: async () => {
    const { wallet } = get();
    if (!wallet.connected) return;

    set({ isLoading: true });
    try {
      const tokens = await heliusService.getWalletBalances(wallet.address);
      const solBalance = tokens.find(t => t.mint === 'So11111111111111111111111111111111111111112')?.uiAmount || 0;

      set({
        wallet: { ...wallet, balance: solBalance, tokens },
        isLoading: false,
      });
    } catch (error) {
      set({ isLoading: false, error: String(error) });
    }
  },

  refreshMarketData: async () => {
    set({ isLoading: true });
    try {
      await marketDataService.refreshData();
      const marketData = marketDataService.getMarketData();
      
      set({
        solPrice: marketData.solPrice,
        coins: marketData.tokens,
        isLoading: false,
      });
    } catch (error) {
      set({ isLoading: false, error: String(error) });
    }
  },
}));