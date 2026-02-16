import { Memecoin } from '../types/trading';
import { SwapResult } from './jupiterSwap';

export interface Position {
  id: string;
  tokenAddress: string;
  symbol: string;
  name: string;
  entryPrice: number;
  currentPrice: number;
  quantity: number;
  decimals: number;
  investedUsd: number;
  currentValue: number;
  pnlUsd: number;
  pnlPercent: number;
  status: 'open' | 'closed';
  entryTime: number;
  exitTime?: number;
  exitPrice?: number;
  exitValue?: number;
  stopLoss?: number;
  takeProfit?: number;
  txSignature?: string;
  exitTxSignature?: string;
  exitReason?: 'stop_loss' | 'take_profit' | 'manual' | 'trailing_stop';
  autoTrade: boolean;
}

export interface PositionConfig {
  defaultStopLossPercent: number; // e.g., 20 = 20% loss
  defaultTakeProfitPercent: number; // e.g., 100 = 100% gain
  trailingStopEnabled: boolean;
  trailingStopPercent: number; // e.g., 10 = 10% from peak
  maxPositions: number;
  maxPositionSizeUsd: number;
}

export class PositionManager {
  private positions: Map<string, Position> = new Map(); // tokenAddress -> Position
  private positionHistory: Position[] = [];
  private config: PositionConfig;
  private onPositionUpdateCallbacks: ((position: Position) => void)[] = [];
  private onPositionCloseCallbacks: ((position: Position) => void)[] = [];
  private onStopLossCallbacks: ((position: Position) => void)[] = [];
  private onTakeProfitCallbacks: ((position: Position) => void)[] = [];

  constructor(config: Partial<PositionConfig> = {}) {
    this.config = {
      defaultStopLossPercent: 20,
      defaultTakeProfitPercent: 100,
      trailingStopEnabled: true,
      trailingStopPercent: 15,
      maxPositions: 10,
      maxPositionSizeUsd: 1000,
      ...config,
    };
    this.loadPositions();
  }

  // Open a new position
  openPosition(
    coin: Memecoin,
    quantity: number,
    entryPrice: number,
    swapResult: SwapResult,
    autoTrade: boolean = false,
    customStopLoss?: number,
    customTakeProfit?: number
  ): Position | null {
    // Check max positions
    if (this.getOpenPositions().length >= this.config.maxPositions) {
      console.warn('Max positions reached');
      return null;
    }

    // Check if position already exists
    if (this.positions.has(coin.address)) {
      console.warn('Position already exists for this token');
      return null;
    }

    const investedUsd = quantity * entryPrice;
    
    // Check max position size
    if (investedUsd > this.config.maxPositionSizeUsd) {
      console.warn('Position size exceeds maximum');
      return null;
    }

    const position: Position = {
      id: `${coin.address}_${Date.now()}`,
      tokenAddress: coin.address,
      symbol: coin.symbol,
      name: coin.name,
      entryPrice,
      currentPrice: entryPrice,
      quantity,
      decimals: coin.decimals,
      investedUsd,
      currentValue: investedUsd,
      pnlUsd: 0,
      pnlPercent: 0,
      status: 'open',
      entryTime: Date.now(),
      stopLoss: customStopLoss || entryPrice * (1 - this.config.defaultStopLossPercent / 100),
      takeProfit: customTakeProfit || entryPrice * (1 + this.config.defaultTakeProfitPercent / 100),
      txSignature: swapResult.signature || undefined,
      autoTrade,
    };

    this.positions.set(coin.address, position);
    this.savePositions();
    this.notifyPositionUpdate(position);

    console.log(`📈 Position opened: ${coin.symbol} @ $${entryPrice.toFixed(6)}`);
    return position;
  }

  // Close a position
  closePosition(
    tokenAddress: string,
    exitPrice: number,
    swapResult: SwapResult,
    reason: Position['exitReason'] = 'manual'
  ): Position | null {
    const position = this.positions.get(tokenAddress);
    if (!position || position.status !== 'open') {
      return null;
    }

    const exitValue = position.quantity * exitPrice;
    const pnlUsd = exitValue - position.investedUsd;
    const pnlPercent = (pnlUsd / position.investedUsd) * 100;

    const closedPosition: Position = {
      ...position,
      status: 'closed',
      exitTime: Date.now(),
      exitPrice,
      exitValue,
      pnlUsd,
      pnlPercent,
      exitTxSignature: swapResult.signature || undefined,
      exitReason: reason,
      currentPrice: exitPrice,
      currentValue: exitValue,
    };

    this.positions.delete(tokenAddress);
    this.positionHistory.push(closedPosition);
    this.savePositions();
    
    this.notifyPositionClose(closedPosition);

    const emoji = pnlUsd >= 0 ? '🟢' : '🔴';
    console.log(`${emoji} Position closed: ${position.symbol} | P&L: $${pnlUsd.toFixed(2)} (${pnlPercent.toFixed(2)}%)`);

    return closedPosition;
  }

  // Update position price (called when price changes)
  updatePrice(tokenAddress: string, currentPrice: number): Position | null {
    const position = this.positions.get(tokenAddress);
    if (!position || position.status !== 'open') {
      return null;
    }

    const currentValue = position.quantity * currentPrice;
    const pnlUsd = currentValue - position.investedUsd;
    const pnlPercent = (pnlUsd / position.investedUsd) * 100;

    // Update trailing stop if enabled
    let stopLoss = position.stopLoss;
    if (this.config.trailingStopEnabled) {
      const peakPrice = position.entryPrice * (1 + Math.max(0, pnlPercent) / 100);
      const trailingStopPrice = peakPrice * (1 - this.config.trailingStopPercent / 100);
      if (trailingStopPrice > (stopLoss || 0)) {
        stopLoss = trailingStopPrice;
      }
    }

    const updatedPosition: Position = {
      ...position,
      currentPrice,
      currentValue,
      pnlUsd,
      pnlPercent,
      stopLoss,
    };

    this.positions.set(tokenAddress, updatedPosition);
    this.notifyPositionUpdate(updatedPosition);

    // Check stop loss / take profit
    this.checkExitConditions(updatedPosition);

    return updatedPosition;
  }

  // Check if position should exit (stop loss / take profit)
  private checkExitConditions(position: Position): void {
    if (position.status !== 'open') return;

    // Check stop loss
    if (position.stopLoss && position.currentPrice <= position.stopLoss) {
      this.onStopLossCallbacks.forEach(cb => {
        try {
          cb(position);
        } catch (e) {
          console.error('Stop loss callback error:', e);
        }
      });
    }

    // Check take profit
    if (position.takeProfit && position.currentPrice >= position.takeProfit) {
      this.onTakeProfitCallbacks.forEach(cb => {
        try {
          cb(position);
        } catch (e) {
          console.error('Take profit callback error:', e);
        }
      });
    }
  }

  // Get open positions
  getOpenPositions(): Position[] {
    return Array.from(this.positions.values())
      .filter(p => p.status === 'open')
      .sort((a, b) => b.entryTime - a.entryTime);
  }

  // Get position history
  getPositionHistory(): Position[] {
    return [...this.positionHistory]
      .sort((a, b) => (b.exitTime || 0) - (a.exitTime || 0));
  }

  // Get position by token address
  getPosition(tokenAddress: string): Position | undefined {
    return this.positions.get(tokenAddress);
  }

  // Check if has open position
  hasPosition(tokenAddress: string): boolean {
    const position = this.positions.get(tokenAddress);
    return position?.status === 'open';
  }

  // Get total P&L
  getTotalPnL(): { realized: number; unrealized: number; total: number } {
    const openPositions = this.getOpenPositions();
    const unrealized = openPositions.reduce((sum, p) => sum + p.pnlUsd, 0);

    const realized = this.positionHistory
      .filter(p => p.status === 'closed')
      .reduce((sum, p) => sum + p.pnlUsd, 0);

    return {
      realized,
      unrealized,
      total: realized + unrealized,
    };
  }

  // Get win rate
  getWinRate(): { wins: number; losses: number; winRate: number } {
    const closed = this.positionHistory.filter(p => p.status === 'closed');
    const wins = closed.filter(p => p.pnlUsd > 0).length;
    const losses = closed.filter(p => p.pnlUsd <= 0).length;
    const total = wins + losses;

    return {
      wins,
      losses,
      winRate: total > 0 ? (wins / total) * 100 : 0,
    };
  }

  // Subscribe to position updates
  onPositionUpdate(callback: (position: Position) => void): () => void {
    this.onPositionUpdateCallbacks.push(callback);
    return () => {
      const index = this.onPositionUpdateCallbacks.indexOf(callback);
      if (index > -1) this.onPositionUpdateCallbacks.splice(index, 1);
    };
  }

  onPositionClose(callback: (position: Position) => void): () => void {
    this.onPositionCloseCallbacks.push(callback);
    return () => {
      const index = this.onPositionCloseCallbacks.indexOf(callback);
      if (index > -1) this.onPositionCloseCallbacks.splice(index, 1);
    };
  }

  onStopLoss(callback: (position: Position) => void): () => void {
    this.onStopLossCallbacks.push(callback);
    return () => {
      const index = this.onStopLossCallbacks.indexOf(callback);
      if (index > -1) this.onStopLossCallbacks.splice(index, 1);
    };
  }

  onTakeProfit(callback: (position: Position) => void): () => void {
    this.onTakeProfitCallbacks.push(callback);
    return () => {
      const index = this.onTakeProfitCallbacks.indexOf(callback);
      if (index > -1) this.onTakeProfitCallbacks.splice(index, 1);
    };
  }

  private notifyPositionUpdate(position: Position): void {
    this.onPositionUpdateCallbacks.forEach(cb => {
      try {
        cb(position);
      } catch (e) {
        console.error('Position update callback error:', e);
      }
    });
  }

  private notifyPositionClose(position: Position): void {
    this.onPositionCloseCallbacks.forEach(cb => {
      try {
        cb(position);
      } catch (e) {
        console.error('Position close callback error:', e);
      }
    });
  }

  // Save positions to localStorage
  private savePositions(): void {
    const data = {
      open: Array.from(this.positions.values()),
      history: this.positionHistory,
      config: this.config,
    };
    localStorage.setItem('positionManager', JSON.stringify(data));
  }

  // Load positions from localStorage
  private loadPositions(): void {
    try {
      const saved = localStorage.getItem('positionManager');
      if (saved) {
        const data = JSON.parse(saved);
        this.positions = new Map(data.open.map((p: Position) => [p.tokenAddress, p]));
        this.positionHistory = data.history || [];
        this.config = { ...this.config, ...data.config };
      }
    } catch (error) {
      console.error('Failed to load positions:', error);
    }
  }

  // Update config
  updateConfig(config: Partial<PositionConfig>): void {
    this.config = { ...this.config, ...config };
    this.savePositions();
  }

  getConfig(): PositionConfig {
    return { ...this.config };
  }
}

export const positionManager = new PositionManager();
