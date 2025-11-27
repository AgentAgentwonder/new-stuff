// Wallet Types
export interface TokenBalance {
  mint: string;
  symbol: string;
  name: string;
  balance: number;
  decimals: number;
  usdValue: number;
  change24h: number;
  logoUri?: string;
  lastUpdated: string;
}

export interface SendTransactionInput {
  recipient: string;
  amount: number;
  tokenMint?: string;
  memo?: string;
}

export interface TransactionFeeEstimate {
  baseFee: number;
  priorityFee: number;
  totalFee: number;
  estimatedUnits: number;
}

export interface QRCodeData {
  address: string;
  amount?: number;
  token?: string;
  label?: string;
  message?: string;
}

export interface SolanaPayQR {
  url: string;
  qrCode: string;
}

// Trading Types
export interface CreateOrderRequest {
  orderType: 'market' | 'limit' | 'stop' | 'stop_limit' | 'trailing_stop';
  side: 'buy' | 'sell';
  inputMint: string;
  outputMint: string;
  inputSymbol: string;
  outputSymbol: string;
  amount: number;
  limitPrice?: number;
  stopPrice?: number;
  trailingPercent?: number;
  linkedOrderId?: string;
  slippageBps?: number;
  priorityFeeMicroLamports?: number;
  walletAddress: string;
}

export interface Order {
  id: string;
  orderType: string;
  side: string;
  status: string;
  inputMint: string;
  outputMint: string;
  inputSymbol: string;
  outputSymbol: string;
  amount: number;
  filledAmount: number;
  limitPrice?: number;
  stopPrice?: number;
  trailingPercent?: number;
  highestPrice?: number;
  lowestPrice?: number;
  linkedOrderId?: string;
  slippageBps?: number;
  priorityFeeMicroLamports?: number;
  walletAddress: string;
  createdAt: string;
  updatedAt: string;
  triggeredAt?: string;
  txSignature?: string;
  errorMessage?: string;
}

// AI Types
export interface ChatMessage {
  role: string;
  content: string;
}

export interface ReasoningStep {
  step: number;
  description: string;
  confidence: number;
}

export interface ChatResponse {
  content: string;
  reasoning?: ReasoningStep[];
  metadata?: Record<string, any>;
}

export interface OptimizationAction {
  type: string;
  token: string;
  amount: number;
  reason: string;
}

export interface PortfolioOptimization {
  id: string;
  timestamp: string;
  currentAllocation: Record<string, number>;
  suggestedAllocation: Record<string, number>;
  expectedReturn: number;
  riskScore: number;
  reasoning: string[];
  actions: OptimizationAction[];
}

export interface PatternWarning {
  id: string;
  timestamp: string;
  pattern: string;
  severity: string;
  tokens: string[];
  description: string;
  recommendation: string;
}

// Portfolio Analytics Types
export interface CorrelationMatrix {
  symbols: string[];
  matrix: number[][];
  calculatedAt: string;
}

export interface DiversificationMetrics {
  score: number;
  effectiveN: number;
  avgCorrelation: number;
  concentrationRisk: number;
}

export interface RiskConcentration {
  symbol: string;
  allocation: number;
  riskLevel: string;
  recommendation: string;
}

export interface SharpeMetrics {
  sharpeRatio: number;
  annualizedReturn: number;
  annualizedVolatility: number;
  riskFreeRate: number;
}

export interface FactorExposure {
  name: string;
  beta: number;
  exposure: number;
}

export interface FactorAnalysis {
  factors: FactorExposure[];
  marketBeta: number;
  systematicRisk: number;
  specificRisk: number;
}

export interface PortfolioAnalytics {
  correlation: CorrelationMatrix;
  diversification: DiversificationMetrics;
  concentration: RiskConcentration[];
  sharpe: SharpeMetrics;
  factors: FactorAnalysis;
  calculatedAt: string;
}

export interface SectorAllocation {
  sector: string;
  allocation: number;
  value: number;
  symbols: string[];
}

// Common Error Types
export interface TauriError {
  code?: string;
  message: string;
  details?: Record<string, any>;
}

// API Response Wrappers
export interface ApiResponse<T> {
  data: T;
  success: boolean;
  error?: TauriError;
}

// Streaming Response Types
export interface StreamingChunk {
  id: string;
  content: string;
  finished: boolean;
  error?: string;
}
