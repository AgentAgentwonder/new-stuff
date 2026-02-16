import { Memecoin, TradeSignal, AITradingConfig } from '../types/trading';
import { RiskCalculator } from './riskCalculator';

interface TradeOutcome {
  id: string;
  coinAddress: string;
  entrySignal: TradeSignal;
  entryPrice: number;
  exitPrice?: number;
  exitTime?: number;
  profitLoss?: number;
  profitLossPercent?: number;
  holdingTimeMinutes?: number;
  exitReason: 'stop_loss' | 'take_profit' | 'manual' | 'timeout' | 'open';
  timestamp: number;
}

interface LearningModel {
  // Pattern weights learned from successful trades
  patternWeights: {
    liquidityScore: number;
    holderScore: number;
    lpBurnedScore: number;
    authorityScore: number;
    priceStabilityScore: number;
    trendMomentumScore: number;
  };
  // Historical performance tracking
  totalTrades: number;
  winningTrades: number;
  losingTrades: number;
  averageWinPercent: number;
  averageLossPercent: number;
  winRate: number;
  // Threshold adjustments based on performance
  adaptiveThresholds: {
    minLiquidityUsd: number;
    minHolders: number;
    minVolume24h: number;
  };
  lastUpdated: number;
}

const DEFAULT_MODEL: LearningModel = {
  patternWeights: {
    liquidityScore: 1.0,
    holderScore: 1.0,
    lpBurnedScore: 1.0,
    authorityScore: 1.0,
    priceStabilityScore: 1.0,
    trendMomentumScore: 1.0,
  },
  totalTrades: 0,
  winningTrades: 0,
  losingTrades: 0,
  averageWinPercent: 0,
  averageLossPercent: 0,
  winRate: 0,
  adaptiveThresholds: {
    minLiquidityUsd: 10000,
    minHolders: 100,
    minVolume24h: 5000,
  },
  lastUpdated: Date.now(),
};

export class AITradingEngine {
  private config: AITradingConfig;
  private riskCalculator: RiskCalculator;
  private dailyTrades: number = 0;
  private lastReset: number = Date.now();
  private learningModel: LearningModel;
  private tradeOutcomes: TradeOutcome[] = [];
  private readonly LEARNING_RATE = 0.1;
  private readonly MAX_HISTORY = 500;

  constructor(config: AITradingConfig, riskCalculator: RiskCalculator) {
    this.config = config;
    this.riskCalculator = riskCalculator;
    this.learningModel = this.loadModel() || DEFAULT_MODEL;
    this.tradeOutcomes = this.loadTradeHistory() || [];
  }

  // Persist learning model to localStorage
  private saveModel(): void {
    if (typeof window !== 'undefined') {
      localStorage.setItem('ai_learning_model', JSON.stringify(this.learningModel));
    }
  }

  private loadModel(): LearningModel | null {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('ai_learning_model');
      return saved ? JSON.parse(saved) : null;
    }
    return null;
  }

  private saveTradeHistory(): void {
    if (typeof window !== 'undefined') {
      localStorage.setItem('ai_trade_history', JSON.stringify(this.tradeOutcomes.slice(-this.MAX_HISTORY)));
    }
  }

  private loadTradeHistory(): TradeOutcome[] | null {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('ai_trade_history');
      return saved ? JSON.parse(saved) : null;
    }
    return null;
  }

  updateConfig(config: AITradingConfig): void {
    this.config = config;
  }

  private resetDailyCounter(): void {
    const now = Date.now();
    const dayMs = 24 * 60 * 60 * 1000;
    if (now - this.lastReset > dayMs) {
      this.dailyTrades = 0;
      this.lastReset = now;
    }
  }

  canTradeToday(): boolean {
    this.resetDailyCounter();
    return this.dailyTrades < this.config.maxDailyTrades;
  }

  // Enhanced signal generation with learned weights
  analyzeCoinWithLearning(coin: Memecoin): TradeSignal & { weightedScore: number } {
    const baseSignal = this.riskCalculator.generateSignal(coin);
    const weights = this.learningModel.patternWeights;
    
    // Calculate weighted confidence score
    let weightedScore = baseSignal.confidence;
    
    if (baseSignal.reasons.includes('Good liquidity')) {
      weightedScore *= weights.liquidityScore;
    }
    if (baseSignal.reasons.includes('Sufficient holders')) {
      weightedScore *= weights.holderScore;
    }
    if (baseSignal.reasons.includes('LP tokens burned')) {
      weightedScore *= weights.lpBurnedScore;
    }
    if (baseSignal.reasons.includes('Authorities disabled')) {
      weightedScore *= weights.authorityScore;
    }
    
    // Normalize to 0-100
    weightedScore = Math.min(100, Math.max(0, weightedScore));
    
    // Adjust signal based on adaptive thresholds
    const adaptiveSignal = this.applyAdaptiveThresholds(baseSignal, coin);
    
    return {
      ...adaptiveSignal,
      confidence: weightedScore,
      weightedScore,
    };
  }

  private applyAdaptiveThresholds(signal: TradeSignal, coin: Memecoin): TradeSignal {
    const thresholds = this.learningModel.adaptiveThresholds;
    
    if (coin.liquidity < thresholds.minLiquidityUsd && signal.signal !== 'red') {
      // Downgrade signal if liquidity is below learned threshold
      return {
        ...signal,
        signal: signal.signal === 'green' ? 'yellow' : 'red',
        reasons: [...signal.reasons, `Liquidity below learned threshold ($${thresholds.minLiquidityUsd.toFixed(0)})`],
      };
    }
    
    return signal;
  }

  analyzeCoin(coin: Memecoin): TradeSignal {
    return this.riskCalculator.generateSignal(coin);
  }

  async analyzeBatch(coins: Memecoin[]): Promise<TradeSignal[]> {
    const signals: TradeSignal[] = [];
    
    for (const coin of coins) {
      const signal = this.analyzeCoin(coin);
      signals.push(signal);
    }

    // Sort by confidence (highest first)
    return signals.sort((a, b) => b.confidence - a.confidence);
  }

  shouldAutoTrade(signal: TradeSignal): boolean {
    if (!this.config.enabled || !this.canTradeToday()) {
      return false;
    }

    if (signal.signal === 'green' && this.config.autoTradeGreen) {
      return true;
    }

    return false;
  }

  shouldNotify(signal: TradeSignal): boolean {
    if (!this.config.enabled) {
      return false;
    }

    if (signal.signal === 'yellow' && this.config.notifyYellow) {
      return true;
    }

    return false;
  }

  // Record a new trade outcome and update learning model
  recordTradeOutcome(outcome: Omit<TradeOutcome, 'timestamp'>): void {
    const fullOutcome: TradeOutcome = {
      ...outcome,
      timestamp: Date.now(),
    };
    
    this.tradeOutcomes.push(fullOutcome);
    
    // Keep only recent history
    if (this.tradeOutcomes.length > this.MAX_HISTORY) {
      this.tradeOutcomes = this.tradeOutcomes.slice(-this.MAX_HISTORY);
    }
    
    this.updateLearningModel(fullOutcome);
    this.saveTradeHistory();
    this.saveModel();
  }

  private updateLearningModel(outcome: TradeOutcome): void {
    const isWin = (outcome.profitLossPercent || 0) > 0;
    
    // Update basic stats
    this.learningModel.totalTrades++;
    if (isWin) {
      this.learningModel.winningTrades++;
      this.learningModel.averageWinPercent = 
        (this.learningModel.averageWinPercent * (this.learningModel.winningTrades - 1) + 
         (outcome.profitLossPercent || 0)) / this.learningModel.winningTrades;
    } else {
      this.learningModel.losingTrades++;
      this.learningModel.averageLossPercent = 
        (this.learningModel.averageLossPercent * (this.learningModel.losingTrades - 1) + 
         Math.abs(outcome.profitLossPercent || 0)) / this.learningModel.losingTrades;
    }
    
    this.learningModel.winRate = this.learningModel.winningTrades / this.learningModel.totalTrades;
    this.learningModel.lastUpdated = Date.now();
    
    // Adjust pattern weights based on trade characteristics
    if (isWin) {
      // Increase weights for factors present in winning trades
      this.adjustWeights(outcome.entrySignal.reasons, 1 + this.LEARNING_RATE);
    } else {
      // Decrease weights for factors present in losing trades
      this.adjustWeights(outcome.entrySignal.reasons, 1 - this.LEARNING_RATE);
    }
    
    // Adapt thresholds based on successful trades
    this.adaptThresholds();
  }

  private adjustWeights(factors: string[], multiplier: number): void {
    // Map factors to weights and adjust
    if (factors.includes('Good liquidity')) {
      this.learningModel.patternWeights.liquidityScore *= multiplier;
    }
    if (factors.includes('Sufficient holders')) {
      this.learningModel.patternWeights.holderScore *= multiplier;
    }
    if (factors.includes('LP tokens burned')) {
      this.learningModel.patternWeights.lpBurnedScore *= multiplier;
    }
    if (factors.includes('Authorities disabled') || factors.includes('Mint authority disabled')) {
      this.learningModel.patternWeights.authorityScore *= multiplier;
    }
    
    // Normalize weights to prevent runaway values
    const weights = this.learningModel.patternWeights;
    const maxWeight = Math.max(...Object.values(weights));
    if (maxWeight > 3) {
      const normalizeFactor = 2 / maxWeight;
      weights.liquidityScore *= normalizeFactor;
      weights.holderScore *= normalizeFactor;
      weights.lpBurnedScore *= normalizeFactor;
      weights.authorityScore *= normalizeFactor;
      weights.priceStabilityScore *= normalizeFactor;
      weights.trendMomentumScore *= normalizeFactor;
    }
  }

  private adaptThresholds(): void {
    // Look at last 50 trades to adapt thresholds
    const recentTrades = this.tradeOutcomes.slice(-50);
    const winningTrades = recentTrades.filter(t => (t.profitLossPercent || 0) > 0);
    
    if (winningTrades.length >= 5) {
      // Calculate average characteristics of winning trades
      const avgLiquidity = winningTrades.reduce((sum, t) => {
        const coin = this.getCoinAtTime(t.coinAddress, t.timestamp);
        return sum + (coin?.liquidity || this.learningModel.adaptiveThresholds.minLiquidityUsd);
      }, 0) / winningTrades.length;
      
      // Gradually move thresholds toward successful patterns
      this.learningModel.adaptiveThresholds.minLiquidityUsd = 
        this.learningModel.adaptiveThresholds.minLiquidityUsd * 0.95 + avgLiquidity * 0.05;
    }
  }

  private getCoinAtTime(_address: string, _timestamp: number): Memecoin | undefined {
    // This would need historical data - for now return undefined
    return undefined;
  }

  // Learn from detected coin patterns (no trade required)
  learnFromPattern(coin: Memecoin, signal: TradeSignal, source: string): void {
    // Create a simulated "observation" outcome
    const observation: TradeOutcome = {
      id: `pattern_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      coinAddress: coin.address,
      entrySignal: signal,
      entryPrice: coin.priceUsd,
      exitPrice: coin.priceUsd, // Assume hold for now
      exitTime: Date.now(),
      profitLoss: 0,
      profitLossPercent: 0,
      holdingTimeMinutes: 0,
      exitReason: 'open',
      timestamp: Date.now(),
    };
    
    this.tradeOutcomes.push(observation);
    
    // Keep only recent history
    if (this.tradeOutcomes.length > this.MAX_HISTORY) {
      this.tradeOutcomes = this.tradeOutcomes.slice(-this.MAX_HISTORY);
    }
    
    // Update pattern recognition based on coin characteristics
    this.updatePatternRecognition(coin, signal, source);
    this.saveTradeHistory();
    
    console.log(`🧠 AI learned pattern from ${source}: ${coin.symbol} (${signal.signal})`);
  }
  
  private updatePatternRecognition(coin: Memecoin, signal: TradeSignal, source: string): void {
    const weights = this.learningModel.patternWeights;
    
    // Boost weights for characteristics of high-confidence signals
    if (signal.confidence > 70) {
      if (coin.liquidity > 10000) weights.liquidityScore = Math.min(3, weights.liquidityScore * 1.02);
      if (coin.holderCount > 100) weights.holderScore = Math.min(3, weights.holderScore * 1.02);
      if (coin.lpBurned) weights.lpBurnedScore = Math.min(3, weights.lpBurnedScore * 1.03);
      if (!coin.mintAuthority && !coin.freezeAuthority) weights.authorityScore = Math.min(3, weights.authorityScore * 1.03);
    }
    
    // Source-specific learning
    if (source === 'helius') {
      // Helius detects very new coins - adjust for early detection
      this.learningModel.adaptiveThresholds.minLiquidityUsd = Math.max(5000, 
        this.learningModel.adaptiveThresholds.minLiquidityUsd * 0.995 + coin.liquidity * 0.005
      );
    }
    
    this.learningModel.lastUpdated = Date.now();
    this.saveModel();
  }

  // Get current learning statistics
  getLearningStats(): LearningModel & { recentPerformance: { last10: number; last50: number } } {
    const recent10 = this.tradeOutcomes.slice(-10);
    const recent50 = this.tradeOutcomes.slice(-50);
    
    const wins10 = recent10.filter(t => (t.profitLossPercent || 0) > 0).length;
    const wins50 = recent50.filter(t => (t.profitLossPercent || 0) > 0).length;
    
    return {
      ...this.learningModel,
      recentPerformance: {
        last10: recent10.length > 0 ? wins10 / recent10.length : 0,
        last50: recent50.length > 0 ? wins50 / recent50.length : 0,
      },
    };
  }

  // Reset learning model
  resetLearning(): void {
    this.learningModel = { ...DEFAULT_MODEL };
    this.tradeOutcomes = [];
    this.saveModel();
    this.saveTradeHistory();
  }

  getDailyStats(): { trades: number; max: number; remaining: number } {
    this.resetDailyCounter();
    return {
      trades: this.dailyTrades,
      max: this.config.maxDailyTrades,
      remaining: Math.max(0, this.config.maxDailyTrades - this.dailyTrades),
    };
  }

  // Advanced analysis using historical data
  async analyzeTrend(_address: string, priceHistory: { price: number; timestamp: number }[]): Promise<{
    trend: 'up' | 'down' | 'sideways';
    volatility: number;
    momentum: number;
  }> {
    if (priceHistory.length < 10) {
      return { trend: 'sideways', volatility: 0, momentum: 0 };
    }

    const prices = priceHistory.map((h) => h.price);
    const recent = prices.slice(-20);
    const older = prices.slice(0, prices.length - 20);

    const recentAvg = recent.reduce((a, b) => a + b, 0) / recent.length;
    const olderAvg = older.length > 0 ? older.reduce((a, b) => a + b, 0) / older.length : recentAvg;

    const trend: 'up' | 'down' | 'sideways' = 
      recentAvg > olderAvg * 1.05 ? 'up' : 
      recentAvg < olderAvg * 0.95 ? 'down' : 'sideways';

    // Calculate volatility
    const mean = recentAvg;
    const squaredDiffs = recent.map((p) => Math.pow(p - mean, 2));
    const volatility = Math.sqrt(squaredDiffs.reduce((a, b) => a + b, 0) / recent.length) / mean * 100;

    // Calculate momentum
    const momentum = ((recent[recent.length - 1] - recent[0]) / recent[0]) * 100;

    return { trend, volatility, momentum };
  }

  // Pattern detection for memecoin-specific behaviors
  detectPumpPattern(coin: Memecoin, transactions: { type: string; valueUsd: number }[]): {
    isPumping: boolean;
    confidence: number;
    warning: string;
  } {
    const recent = transactions.slice(-50);
    const buyPressure = recent.filter((t) => t.type === 'buy').reduce((a, b) => a + b.valueUsd, 0);
    const sellPressure = recent.filter((t) => t.type === 'sell').reduce((a, b) => a + b.valueUsd, 0);
    
    const ratio = sellPressure > 0 ? buyPressure / sellPressure : buyPressure;
    
    if (ratio > 3 && coin.priceChange24h > 50) {
      return {
        isPumping: true,
        confidence: 85,
        warning: 'Potential pump detected - exercise caution',
      };
    }

    if (coin.priceChange24h > 100 && coin.volume24h > coin.marketCap * 0.5) {
      return {
        isPumping: true,
        confidence: 75,
        warning: 'High volume relative to market cap',
      };
    }

    return {
      isPumping: false,
      confidence: 50,
      warning: '',
    };
  }
}
