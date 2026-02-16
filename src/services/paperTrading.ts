import { Memecoin, TradeSignal } from '../types/trading';
import { Position, PositionManager } from './positionManager';

export interface PaperTrade {
  id: string;
  tokenAddress: string;
  symbol: string;
  type: 'buy' | 'sell';
  amount: number;
  price: number;
  total: number;
  timestamp: number;
  reason?: string;
  aiSignal?: 'green' | 'yellow' | 'red';
  confidence?: number;
}

export interface PaperPortfolio {
  balance: number; // In SOL
  positions: Position[];
  trades: PaperTrade[];
  totalValue: number;
  totalPnl: number;
  totalPnlPercent: number;
  winRate: number;
  tradesCount: number;
}

export class PaperTradingService {
  private balance: number = 10; // Start with 10 SOL
  private positions: Map<string, Position> = new Map();
  private trades: PaperTrade[] = [];
  private startingBalance: number = 10;
  private positionManager: PositionManager;
  private onTradeCallbacks: ((trade: PaperTrade) => void)[] = [];
  private onBalanceUpdateCallbacks: ((balance: number) => void)[] = [];

  constructor() {
    this.positionManager = new PositionManager({
      maxPositions: 10,
      maxPositionSizeUsd: 1, // 1 SOL max per trade
      defaultStopLossPercent: 20,
      defaultTakeProfitPercent: 100,
    });
    this.loadData();
  }

  // Get current balance
  getBalance(): number {
    return this.balance;
  }

  // Set starting balance
  setStartingBalance(amount: number): void {
    this.startingBalance = amount;
    if (this.trades.length === 0) {
      this.balance = amount;
    }
    this.saveData();
  }

  // Reset paper trading
  reset(): void {
    this.balance = this.startingBalance;
    this.positions.clear();
    this.trades = [];
    this.saveData();
    console.log('🔄 Paper trading reset. Balance:', this.balance, 'SOL');
  }

  // Simulate buying a token
  buy(coin: Memecoin, solAmount: number, price: number, aiSignal?: TradeSignal): { success: boolean; trade?: PaperTrade; error?: string } {
    // Check balance
    if (solAmount > this.balance) {
      return { success: false, error: 'Insufficient balance' };
    }

    // Check if already have position
    if (this.positions.has(coin.address)) {
      return { success: false, error: 'Position already exists' };
    }

    // Calculate token amount
    const tokenAmount = solAmount / price;

    // Deduct balance
    this.balance -= solAmount;

    // Create position
    const position: Position = {
      id: `${coin.address}_${Date.now()}`,
      tokenAddress: coin.address,
      symbol: coin.symbol,
      name: coin.name,
      entryPrice: price,
      currentPrice: price,
      quantity: tokenAmount,
      decimals: coin.decimals,
      investedUsd: solAmount,
      currentValue: solAmount,
      pnlUsd: 0,
      pnlPercent: 0,
      status: 'open',
      entryTime: Date.now(),
      stopLoss: price * 0.8, // 20% stop loss
      takeProfit: price * 2, // 100% take profit
      autoTrade: false,
    };

    this.positions.set(coin.address, position);

    // Record trade
    const trade: PaperTrade = {
      id: `paper_${Date.now()}`,
      tokenAddress: coin.address,
      symbol: coin.symbol,
      type: 'buy',
      amount: tokenAmount,
      price,
      total: solAmount,
      timestamp: Date.now(),
      reason: 'Manual paper trade',
      aiSignal: aiSignal?.signal,
      confidence: aiSignal?.confidence,
    };

    this.trades.push(trade);
    this.saveData();
    this.notifyTrade(trade);
    this.notifyBalanceUpdate(this.balance);

    console.log(`📘 Paper Buy: ${coin.symbol} | ${tokenAmount.toFixed(4)} tokens @ $${price.toFixed(6)} | Cost: ${solAmount.toFixed(4)} SOL`);

    return { success: true, trade };
  }

  // Simulate selling a token
  sell(tokenAddress: string, price: number, reason: string = 'Manual'): { success: boolean; trade?: PaperTrade; pnl?: number; error?: string } {
    const position = this.positions.get(tokenAddress);
    if (!position) {
      return { success: false, error: 'Position not found' };
    }

    const sellValue = position.quantity * price;
    const pnl = sellValue - position.investedUsd;
    const pnlPercent = (pnl / position.investedUsd) * 100;

    // Add to balance
    this.balance += sellValue;

    // Record trade
    const trade: PaperTrade = {
      id: `paper_${Date.now()}`,
      tokenAddress: position.tokenAddress,
      symbol: position.symbol,
      type: 'sell',
      amount: position.quantity,
      price,
      total: sellValue,
      timestamp: Date.now(),
      reason,
    };

    this.trades.push(trade);

    // Remove position
    this.positions.delete(tokenAddress);

    this.saveData();
    this.notifyTrade(trade);
    this.notifyBalanceUpdate(this.balance);

    const emoji = pnl >= 0 ? '🟢' : '🔴';
    console.log(`${emoji} Paper Sell: ${position.symbol} @ $${price.toFixed(6)} | P&L: ${pnl.toFixed(4)} SOL (${pnlPercent.toFixed(2)}%)`);

    return { success: true, trade, pnl };
  }

  // Auto-sell based on AI signal
  autoSell(tokenAddress: string, price: number, signal: TradeSignal): { success: boolean; trade?: PaperTrade; pnl?: number } {
    if (signal.signal === 'red') {
      return this.sell(tokenAddress, price, 'AI Red Signal');
    }
    return { success: false };
  }

  // Update position prices (called periodically)
  updatePrices(priceUpdates: { address: string; price: number }[]): void {
    for (const update of priceUpdates) {
      const position = this.positions.get(update.address);
      if (position) {
        const currentValue = position.quantity * update.price;
        const pnl = currentValue - position.investedUsd;
        const pnlPercent = (pnl / position.investedUsd) * 100;

        // Update trailing stop
        const peakPrice = Math.max(position.entryPrice, update.price);
        const trailingStop = peakPrice * 0.85; // 15% trailing stop

        position.currentPrice = update.price;
        position.currentValue = currentValue;
        position.pnlUsd = pnl;
        position.pnlPercent = pnlPercent;
        
        if (trailingStop > (position.stopLoss || 0)) {
          position.stopLoss = trailingStop;
        }

        // Check stop loss / take profit
        if (position.stopLoss && update.price <= position.stopLoss) {
          this.sell(update.address, update.price, 'Stop Loss Triggered');
        } else if (position.takeProfit && update.price >= position.takeProfit) {
          this.sell(update.address, update.price, 'Take Profit Triggered');
        }
      }
    }
  }

  // Get portfolio summary
  getPortfolio(): PaperPortfolio {
    const positions = Array.from(this.positions.values());
    
    // Calculate total value
    const positionsValue = positions.reduce((sum, p) => sum + p.currentValue, 0);
    const totalValue = this.balance + positionsValue;

    // Calculate P&L
    const totalPnl = totalValue - this.startingBalance;
    const totalPnlPercent = (totalPnl / this.startingBalance) * 100;

    // Calculate win rate
    const sells = this.trades.filter(t => t.type === 'sell');
    const winningSells = sells.filter(t => {
      const matchingBuy = this.trades.find(b => b.tokenAddress === t.tokenAddress && b.type === 'buy');
      if (!matchingBuy) return false;
      return t.total > matchingBuy.total;
    });
    const winRate = sells.length > 0 ? (winningSells.length / sells.length) * 100 : 0;

    return {
      balance: this.balance,
      positions,
      trades: [...this.trades],
      totalValue,
      totalPnl,
      totalPnlPercent,
      winRate,
      tradesCount: this.trades.length,
    };
  }

  // Get open positions
  getOpenPositions(): Position[] {
    return Array.from(this.positions.values());
  }

  // Get trade history
  getTradeHistory(): PaperTrade[] {
    return [...this.trades].sort((a, b) => b.timestamp - a.timestamp);
  }

  // Get performance stats
  getStats(): {
    totalTrades: number;
    winningTrades: number;
    losingTrades: number;
    winRate: number;
    avgProfit: number;
    avgLoss: number;
    profitFactor: number;
    bestTrade: PaperTrade | null;
    worstTrade: PaperTrade | null;
  } {
    const sells = this.trades.filter(t => t.type === 'sell');
    const buys = this.trades.filter(t => t.type === 'buy');

    let winningTrades = 0;
    let losingTrades = 0;
    let totalProfit = 0;
    let totalLoss = 0;
    let bestTrade: PaperTrade | null = null;
    let worstTrade: PaperTrade | null = null;
    let bestPnl = -Infinity;
    let worstPnl = Infinity;

    for (const sell of sells) {
      const buy = buys.find(b => b.tokenAddress === sell.tokenAddress);
      if (!buy) continue;

      const pnl = sell.total - buy.total;

      if (pnl > 0) {
        winningTrades++;
        totalProfit += pnl;
      } else {
        losingTrades++;
        totalLoss += Math.abs(pnl);
      }

      if (pnl > bestPnl) {
        bestPnl = pnl;
        bestTrade = sell;
      }
      if (pnl < worstPnl) {
        worstPnl = pnl;
        worstTrade = sell;
      }
    }

    const totalTrades = winningTrades + losingTrades;

    return {
      totalTrades,
      winningTrades,
      losingTrades,
      winRate: totalTrades > 0 ? (winningTrades / totalTrades) * 100 : 0,
      avgProfit: winningTrades > 0 ? totalProfit / winningTrades : 0,
      avgLoss: losingTrades > 0 ? totalLoss / losingTrades : 0,
      profitFactor: totalLoss > 0 ? totalProfit / totalLoss : 0,
      bestTrade,
      worstTrade,
    };
  }

  // Subscribe to trade events
  onTrade(callback: (trade: PaperTrade) => void): () => void {
    this.onTradeCallbacks.push(callback);
    return () => {
      const index = this.onTradeCallbacks.indexOf(callback);
      if (index > -1) this.onTradeCallbacks.splice(index, 1);
    };
  }

  onBalanceUpdate(callback: (balance: number) => void): () => void {
    this.onBalanceUpdateCallbacks.push(callback);
    return () => {
      const index = this.onBalanceUpdateCallbacks.indexOf(callback);
      if (index > -1) this.onBalanceUpdateCallbacks.splice(index, 1);
    };
  }

  private notifyTrade(trade: PaperTrade): void {
    this.onTradeCallbacks.forEach(cb => {
      try {
        cb(trade);
      } catch (e) {
        console.error('Trade callback error:', e);
      }
    });
  }

  private notifyBalanceUpdate(balance: number): void {
    this.onBalanceUpdateCallbacks.forEach(cb => {
      try {
        cb(balance);
      } catch (e) {
        console.error('Balance update callback error:', e);
      }
    });
  }

  private saveData(): void {
    const data = {
      balance: this.balance,
      startingBalance: this.startingBalance,
      positions: Array.from(this.positions.values()),
      trades: this.trades,
    };
    localStorage.setItem('paperTrading', JSON.stringify(data));
  }

  private loadData(): void {
    try {
      const saved = localStorage.getItem('paperTrading');
      if (saved) {
        const data = JSON.parse(saved);
        this.balance = data.balance || 10;
        this.startingBalance = data.startingBalance || 10;
        this.positions = new Map(data.positions?.map((p: Position) => [p.tokenAddress, p]) || []);
        this.trades = data.trades || [];
      }
    } catch (error) {
      console.error('Failed to load paper trading data:', error);
    }
  }
}

export const paperTradingService = new PaperTradingService();
