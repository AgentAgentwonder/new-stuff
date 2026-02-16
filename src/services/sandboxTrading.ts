import { Memecoin, TradeSignal, TradeExecution, Position } from '../types/trading';
import { AITradingEngine } from './aiTrading';
import { RiskCalculator } from './riskCalculator';

export interface SandboxPosition extends Position {
  isSandbox: true;
  exitPrice?: number;
  exitTime?: number;
  exitReason?: 'stop_loss' | 'take_profit' | 'manual' | 'timeout';
}

export interface SandboxTradeOutcome {
  id: string;
  coinAddress: string;
  entrySignal: TradeSignal;
  entryPrice: number;
  exitPrice: number;
  profitLoss: number;
  profitLossPercent: number;
  holdingTimeMinutes: number;
  exitReason: 'stop_loss' | 'take_profit' | 'manual' | 'timeout';
  timestamp: number;
  isWin: boolean;
}

export interface SandboxPortfolio {
  initialBalance: number;
  currentBalance: number;
  totalProfitLoss: number;
  totalProfitLossPercent: number;
  openPositions: SandboxPosition[];
  closedTrades: SandboxTradeOutcome[];
  winRate: number;
  totalTrades: number;
  winningTrades: number;
  losingTrades: number;
}

export interface SandboxConfig {
  initialBalance: number;
  autoSimulate: boolean;
  simulationSpeed: 'realtime' | 'fast' | 'instant'; // How fast to simulate price movements
  maxOpenPositions: number;
  stopLossPercent: number;
  takeProfitPercent: number;
  maxHoldTimeHours: number; // Auto-close after this time
}

const DEFAULT_SANDBOX_CONFIG: SandboxConfig = {
  initialBalance: 10000, // $10K virtual money
  autoSimulate: true,
  simulationSpeed: 'realtime',
  maxOpenPositions: 5,
  stopLossPercent: 15,
  takeProfitPercent: 50,
  maxHoldTimeHours: 24,
};

export class SandboxTradingService {
  private config: SandboxConfig;
  private portfolio: SandboxPortfolio;
  private aiEngine: AITradingEngine;
  private simulationIntervals: Map<string, number> = new Map();
  private isRunning: boolean = false;

  constructor(
    aiEngine: AITradingEngine,
    config: Partial<SandboxConfig> = {}
  ) {
    this.config = { ...DEFAULT_SANDBOX_CONFIG, ...config };
    this.aiEngine = aiEngine;
    this.portfolio = this.loadPortfolio() || this.createInitialPortfolio();
  }

  private createInitialPortfolio(): SandboxPortfolio {
    return {
      initialBalance: this.config.initialBalance,
      currentBalance: this.config.initialBalance,
      totalProfitLoss: 0,
      totalProfitLossPercent: 0,
      openPositions: [],
      closedTrades: [],
      winRate: 0,
      totalTrades: 0,
      winningTrades: 0,
      losingTrades: 0,
    };
  }

  // Persist portfolio to localStorage
  private savePortfolio(): void {
    if (typeof window !== 'undefined') {
      localStorage.setItem('sandbox_portfolio', JSON.stringify(this.portfolio));
    }
  }

  private loadPortfolio(): SandboxPortfolio | null {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('sandbox_portfolio');
      if (saved) {
        const parsed = JSON.parse(saved);
        // Restore methods won't work on parsed objects, but data is there
        return parsed;
      }
    }
    return null;
  }

  // Reset sandbox to initial state
  reset(): void {
    this.portfolio = this.createInitialPortfolio();
    this.savePortfolio();
    this.stopAllSimulations();
  }

  getPortfolio(): SandboxPortfolio {
    return { ...this.portfolio };
  }

  getConfig(): SandboxConfig {
    return { ...this.config };
  }

  updateConfig(config: Partial<SandboxConfig>): void {
    this.config = { ...this.config, ...config };
  }

  // Execute a virtual buy trade
  async executeVirtualBuy(coin: Memecoin, signal: TradeSignal): Promise<TradeExecution | null> {
    // Check if we have enough balance
    const positionSize = Math.min(
      this.portfolio.currentBalance * 0.2, // Max 20% per position
      signal.recommendedPosition * 1000 // Scale up for sandbox
    );

    if (positionSize > this.portfolio.currentBalance) {
      console.log('Insufficient virtual balance');
      return null;
    }

    if (this.portfolio.openPositions.length >= this.config.maxOpenPositions) {
      console.log('Max open positions reached');
      return null;
    }

    // Check if already have position in this coin
    if (this.portfolio.openPositions.some(p => p.coin.address === coin.address)) {
      console.log('Already have position in this coin');
      return null;
    }

    const amount = positionSize / coin.priceUsd;
    const totalUsd = positionSize;

    const execution: TradeExecution = {
      id: `sandbox-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      coinAddress: coin.address,
      type: 'buy',
      amount,
      priceUsd: coin.priceUsd,
      totalUsd,
      slippage: 0.5,
      status: 'confirmed',
      timestamp: Date.now(),
      executedAt: Date.now(),
    };

    // Create position
    const position: SandboxPosition = {
      coin,
      amount,
      entryPrice: coin.priceUsd,
      currentPrice: coin.priceUsd,
      valueUsd: totalUsd,
      pnl: 0,
      pnlPercentage: 0,
      openedAt: Date.now(),
      isSandbox: true,
    };

    this.portfolio.openPositions.push(position);
    this.portfolio.currentBalance -= totalUsd;

    // Start price simulation for this position
    this.startPriceSimulation(coin);

    this.savePortfolio();

    // Record to AI for daily trade tracking
    // (learning happens on exit, not entry)

    return execution;
  }

  // Execute a virtual sell trade
  executeVirtualSell(coinAddress: string, reason: 'stop_loss' | 'take_profit' | 'manual' | 'timeout'): TradeExecution | null {
    const positionIndex = this.portfolio.openPositions.findIndex(p => p.coin.address === coinAddress);
    if (positionIndex === -1) {
      console.log('Position not found');
      return null;
    }

    const position = this.portfolio.openPositions[positionIndex];
    const currentPrice = position.coin.priceUsd;
    const exitValue = position.amount * currentPrice;
    const profitLoss = exitValue - (position.amount * position.entryPrice);
    const profitLossPercent = (profitLoss / (position.amount * position.entryPrice)) * 100;
    const holdingTimeMinutes = (Date.now() - position.openedAt) / (1000 * 60);

    const execution: TradeExecution = {
      id: `sandbox-sell-${Date.now()}`,
      coinAddress,
      type: 'sell',
      amount: position.amount,
      priceUsd: currentPrice,
      totalUsd: exitValue,
      slippage: 0.5,
      status: 'confirmed',
      timestamp: Date.now(),
      executedAt: Date.now(),
    };

    // Record outcome
    const outcome: SandboxTradeOutcome = {
      id: execution.id,
      coinAddress,
      entrySignal: {
        coin: position.coin,
        signal: 'green', // Assume it was a green signal when entered
        confidence: 75,
        reasons: ['Sandbox entry'],
        riskScore: 5,
        potentialReturn: profitLossPercent,
        recommendedPosition: position.valueUsd,
        timestamp: position.openedAt,
      },
      entryPrice: position.entryPrice,
      exitPrice: currentPrice,
      profitLoss,
      profitLossPercent,
      holdingTimeMinutes,
      exitReason: reason,
      timestamp: Date.now(),
      isWin: profitLoss > 0,
    };

    this.portfolio.closedTrades.push(outcome);
    this.portfolio.currentBalance += exitValue;
    this.portfolio.totalProfitLoss += profitLoss;
    this.portfolio.totalProfitLossPercent = (this.portfolio.totalProfitLoss / this.portfolio.initialBalance) * 100;
    this.portfolio.totalTrades++;

    if (profitLoss > 0) {
      this.portfolio.winningTrades++;
    } else {
      this.portfolio.losingTrades++;
    }

    this.portfolio.winRate = this.portfolio.winningTrades / this.portfolio.totalTrades;

    // Remove position
    this.portfolio.openPositions.splice(positionIndex, 1);

    // Stop simulation for this coin if no other positions
    if (!this.portfolio.openPositions.some(p => p.coin.address === coinAddress)) {
      this.stopPriceSimulation(coinAddress);
    }

    this.savePortfolio();

    // Feed outcome to AI for learning
    this.aiEngine.recordTradeOutcome({
      id: outcome.id,
      coinAddress: outcome.coinAddress,
      entrySignal: outcome.entrySignal,
      entryPrice: outcome.entryPrice,
      exitPrice: outcome.exitPrice,
      profitLoss: outcome.profitLoss,
      profitLossPercent: outcome.profitLossPercent,
      holdingTimeMinutes: outcome.holdingTimeMinutes,
      exitReason: outcome.exitReason,
    });

    return execution;
  }

  // Start simulating price movements for a coin
  private startPriceSimulation(coin: Memecoin): void {
    if (this.simulationIntervals.has(coin.address)) {
      return; // Already simulating
    }

    const interval = window.setInterval(() => {
      this.simulatePriceMovement(coin);
    }, this.getSimulationInterval());

    this.simulationIntervals.set(coin.address, interval);
  }

  private stopPriceSimulation(coinAddress: string): void {
    const interval = this.simulationIntervals.get(coinAddress);
    if (interval) {
      window.clearInterval(interval);
      this.simulationIntervals.delete(coinAddress);
    }
  }

  private stopAllSimulations(): void {
    this.simulationIntervals.forEach(interval => window.clearInterval(interval));
    this.simulationIntervals.clear();
  }

  private getSimulationInterval(): number {
    switch (this.config.simulationSpeed) {
      case 'instant':
        return 100; // Fast updates
      case 'fast':
        return 1000; // 1 second
      case 'realtime':
      default:
        return 5000; // 5 seconds (faster than real for simulation)
    }
  }

  // Simulate realistic memecoin price movements
  private simulatePriceMovement(coin: Memecoin): void {
    // Find position for this coin
    const position = this.portfolio.openPositions.find(p => p.coin.address === coin.address);
    if (!position) return;

    // Generate realistic price change based on memecoin volatility
    // Memecoins are highly volatile - can move 5-50% in minutes
    const volatility = 0.05; // 5% base volatility per update
    const drift = position.coin.priceChange24h / 100 / (24 * 12); // Trend toward 24h change
    
    const randomChange = (Math.random() - 0.5) * 2 * volatility;
    const priceChange = drift + randomChange;
    
    const newPrice = position.coin.priceUsd * (1 + priceChange);
    
    // Update coin price
    position.coin.priceUsd = Math.max(newPrice, 0.00000001); // Min price
    position.currentPrice = position.coin.priceUsd;
    
    // Update position P&L
    const currentValue = position.amount * position.currentPrice;
    const investedValue = position.amount * position.entryPrice;
    position.pnl = currentValue - investedValue;
    position.pnlPercentage = (position.pnl / investedValue) * 100;
    position.valueUsd = currentValue;

    // Check stop loss / take profit
    this.checkExitConditions(position);

    this.savePortfolio();
  }

  private checkExitConditions(position: SandboxPosition): void {
    const pnlPercent = position.pnlPercentage;

    // Check stop loss
    if (pnlPercent <= -this.config.stopLossPercent) {
      this.executeVirtualSell(position.coin.address, 'stop_loss');
      return;
    }

    // Check take profit
    if (pnlPercent >= this.config.takeProfitPercent) {
      this.executeVirtualSell(position.coin.address, 'take_profit');
      return;
    }

    // Check max hold time
    const holdTimeHours = (Date.now() - position.openedAt) / (1000 * 60 * 60);
    if (holdTimeHours >= this.config.maxHoldTimeHours) {
      this.executeVirtualSell(position.coin.address, 'timeout');
      return;
    }
  }

  // Auto-trading in sandbox mode
  startAutoTrading(coins: Memecoin[]): void {
    if (this.isRunning) return;
    this.isRunning = true;

    console.log('Starting sandbox auto-trading...');

    // Analyze coins and auto-trade on green signals
    const interval = window.setInterval(async () => {
      if (!this.isRunning) {
        window.clearInterval(interval);
        return;
      }

      for (const coin of coins) {
        // Skip if already have position
        if (this.portfolio.openPositions.some(p => p.coin.address === coin.address)) {
          continue;
        }

        // Skip if no balance
        if (this.portfolio.currentBalance < 100) {
          continue;
        }

        const signal = this.aiEngine.analyzeCoinWithLearning(coin);
        
        if (signal.signal === 'green' && this.aiEngine.shouldAutoTrade(signal)) {
          console.log(`Sandbox auto-buy: ${coin.symbol} at $${coin.priceUsd}`);
          await this.executeVirtualBuy(coin, signal);
        }
      }
    }, 10000); // Check every 10 seconds

    this.simulationIntervals.set('auto-trader', interval);
  }

  stopAutoTrading(): void {
    this.isRunning = false;
    const interval = this.simulationIntervals.get('auto-trader');
    if (interval) {
      window.clearInterval(interval);
      this.simulationIntervals.delete('auto-trader');
    }
  }

  // Get performance analytics
  getAnalytics(): {
    totalReturn: number;
    winRate: number;
    avgWin: number;
    avgLoss: number;
    profitFactor: number;
    avgHoldTime: number;
    bestTrade: SandboxTradeOutcome | null;
    worstTrade: SandboxTradeOutcome | null;
  } {
    const closed = this.portfolio.closedTrades;
    
    if (closed.length === 0) {
      return {
        totalReturn: 0,
        winRate: 0,
        avgWin: 0,
        avgLoss: 0,
        profitFactor: 0,
        avgHoldTime: 0,
        bestTrade: null,
        worstTrade: null,
      };
    }

    const wins = closed.filter(t => t.isWin);
    const losses = closed.filter(t => !t.isWin);

    const avgWin = wins.length > 0 
      ? wins.reduce((sum, t) => sum + t.profitLossPercent, 0) / wins.length 
      : 0;
    
    const avgLoss = losses.length > 0 
      ? losses.reduce((sum, t) => sum + Math.abs(t.profitLossPercent), 0) / losses.length 
      : 0;

    const totalWinAmount = wins.reduce((sum, t) => sum + t.profitLoss, 0);
    const totalLossAmount = losses.reduce((sum, t) => sum + Math.abs(t.profitLoss), 0);

    const sortedByProfit = [...closed].sort((a, b) => b.profitLoss - a.profitLoss);

    return {
      totalReturn: this.portfolio.totalProfitLossPercent,
      winRate: this.portfolio.winRate,
      avgWin,
      avgLoss,
      profitFactor: totalLossAmount > 0 ? totalWinAmount / totalLossAmount : totalWinAmount,
      avgHoldTime: closed.reduce((sum, t) => sum + t.holdingTimeMinutes, 0) / closed.length,
      bestTrade: sortedByProfit[0] || null,
      worstTrade: sortedByProfit[sortedByProfit.length - 1] || null,
    };
  }

  // Manual close all positions
  closeAllPositions(reason: 'manual' | 'timeout' = 'manual'): void {
    const positions = [...this.portfolio.openPositions];
    for (const position of positions) {
      this.executeVirtualSell(position.coin.address, reason);
    }
  }
}
