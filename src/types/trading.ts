export interface Memecoin {
  address: string;
  symbol: string;
  name: string;
  decimals: number;
  priceUsd: number;
  priceChange24h: number;
  volume24h: number;
  marketCap: number;
  liquidity: number;
  holderCount: number;
  createdAt: number;
  isVerified: boolean;
  isMutable: boolean;
  mintAuthority: string | null;
  freezeAuthority: string | null;
  supply: number;
  circulatingSupply: number;
  lpBurned: boolean;
  topHolders: HolderInfo[];
  recentTransactions: Transaction[];
  lastUpdated: number;
}

export interface HolderInfo {
  address: string;
  balance: number;
  percentage: number;
}

export interface Transaction {
  signature: string;
  timestamp: number;
  type: 'buy' | 'sell';
  amount: number;
  priceUsd: number;
  valueUsd: number;
  wallet: string;
}

export interface PriceUpdate {
  address: string;
  priceUsd: number;
  timestamp: number;
  source: string;
}

export interface TradeSignal {
  coin: Memecoin;
  signal: 'green' | 'yellow' | 'red';
  confidence: number;
  reasons: string[];
  riskScore: number;
  potentialReturn: number;
  recommendedPosition: number;
  timestamp: number;
}

export interface TradeExecution {
  id: string;
  coinAddress: string;
  type: 'buy' | 'sell';
  amount: number;
  priceUsd: number;
  totalUsd: number;
  slippage: number;
  status: 'pending' | 'confirmed' | 'failed';
  signature?: string;
  error?: string;
  timestamp: number;
  executedAt?: number;
}

export interface RiskConfig {
  maxPositionSize: number;
  maxSlippage: number;
  minLiquidity: number;
  minHolderCount: number;
  maxSingleTrade: number;
  stopLossPercentage: number;
  takeProfitPercentage: number;
  greenThreshold: number;
  yellowThreshold: number;
  notifyYellow: boolean;
}

export interface AITradingConfig {
  enabled: boolean;
  autoTradeGreen: boolean;
  notifyYellow: boolean;
  maxDailyTrades: number;
  model: string;
  trainingData: string[];
}

export interface WalletState {
  connected: boolean;
  address: string | null;
  balance: number;
  tokenBalances: TokenBalance[];
}

export interface TokenBalance {
  mint: string;
  symbol: string;
  balance: number;
  valueUsd: number;
}

export interface PortfolioState {
  totalValue: number;
  positions: Position[];
  pnl24h: number;
  pnlTotal: number;
}

export interface Position {
  coin: Memecoin;
  amount: number;
  entryPrice: number;
  currentPrice: number;
  valueUsd: number;
  pnl: number;
  pnlPercentage: number;
  openedAt: number;
}
