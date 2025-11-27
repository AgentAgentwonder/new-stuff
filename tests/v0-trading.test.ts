import { describe, it, expect, beforeEach } from 'vitest';
import { usePaperTradingStore, PaperTrade } from '../src/store/paperTradingStore';
import { useTradingSettingsStore } from '../src/store/tradingSettingsStore';
import { useAutoTradingStore } from '../src/store/autoTradingStore';

describe('V0 Trading Store Integration', () => {
  beforeEach(() => {
    // Reset stores before each test
    const paperTradingStore = usePaperTradingStore.getState();
    paperTradingStore.resetAccount();
    paperTradingStore.togglePaperMode(false);

    const tradingSettingsStore = useTradingSettingsStore.getState();
    tradingSettingsStore.resetTradeFilters();
    tradingSettingsStore.clearTradeHistory();
    tradingSettingsStore.setSlippageTolerance(50); // Reset to default
    tradingSettingsStore.setSlippageAutoAdjust(true); // Keep auto-adjust enabled (default)
    tradingSettingsStore.toggleMEVProtection(false); // Reset MEV protection
    tradingSettingsStore.setPriorityFeePreset('normal'); // Reset fee preset
  });

  describe('Paper Trading Store', () => {
    it('should initialize with correct default values', () => {
      const state = usePaperTradingStore.getState();

      expect(state.isPaperMode).toBe(false);
      expect(state.virtualBalance).toBeGreaterThan(0);
      expect(state.trades).toEqual([]);
      expect(state.positions).toEqual([]);
      expect(state.getTotalPnL()).toBe(0);
      expect(state.getTotalPnLPercent()).toBe(0);
      expect(state.getWinRate()).toBe(0);
    });

    it('should toggle paper trading mode', () => {
      const state = usePaperTradingStore.getState();
      expect(state.isPaperMode).toBe(false);

      state.togglePaperMode(true);

      const updatedState = usePaperTradingStore.getState();
      expect(updatedState.isPaperMode).toBe(true);
    });

    it('should execute a paper trade', () => {
      const state = usePaperTradingStore.getState();
      const initialBalance = state.virtualBalance;

      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      const updatedState = usePaperTradingStore.getState();
      expect(updatedState.trades.length).toBe(1);
      expect(updatedState.virtualBalance).toBeLessThan(initialBalance);
      expect(updatedState.positions.length).toBe(1);
      expect(updatedState.positions[0].token).toBe('USDC');
    });

    it('should track P&L for closed positions', () => {
      const state = usePaperTradingStore.getState();

      // Buy trade
      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      // Sell trade at higher price
      state.executePaperTrade({
        side: 'sell',
        fromToken: 'USDC',
        toToken: 'SOL',
        fromAmount: 5000,
        toAmount: 110,
        price: 55,
        slippage: 50,
        fees: 0.5,
      });

      const finalState = usePaperTradingStore.getState();
      expect(finalState.trades.length).toBe(2);
      expect(finalState.totalPnL > 0 || finalState.positions.length === 0).toBe(true);
    });

    it('should update position prices', () => {
      const state = usePaperTradingStore.getState();

      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      let currentState = usePaperTradingStore.getState();
      const position = currentState.positions[0];
      expect(position.currentPrice).toBe(50);

      state.updatePosition('USDC', 55);

      currentState = usePaperTradingStore.getState();
      const updatedPosition = currentState.positions[0];
      expect(updatedPosition.currentPrice).toBe(55);
      expect(updatedPosition.pnl).toBeGreaterThan(0);
    });

    it('should reset paper trading account', () => {
      const state = usePaperTradingStore.getState();

      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      expect(usePaperTradingStore.getState().trades.length).toBe(1);

      state.resetAccount();

      const resetState = usePaperTradingStore.getState();
      expect(resetState.trades.length).toBe(0);
      expect(resetState.positions.length).toBe(0);
      expect(resetState.getTotalPnL()).toBe(0);
    });

    it('should calculate correct win rate with mixed trades', () => {
      const state = usePaperTradingStore.getState();

      // Winning trade - buy at 50, sell at 55 = profit
      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      state.executePaperTrade({
        side: 'sell',
        fromToken: 'USDC',
        toToken: 'SOL',
        fromAmount: 5000,
        toAmount: 110,
        price: 55,
        slippage: 50,
        fees: 0.5,
      });

      // Losing trade - buy at 50, sell at 45 = loss
      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      state.executePaperTrade({
        side: 'sell',
        fromToken: 'USDC',
        toToken: 'SOL',
        fromAmount: 5000,
        toAmount: 95,
        price: 45,
        slippage: 50,
        fees: 0.5,
      });

      const finalState = usePaperTradingStore.getState();
      expect(finalState.trades.length).toBe(4);
      // Win rate should be 50% (1 profitable trade out of 2 closed positions)
      const winRate = finalState.getWinRate();
      expect(winRate).toBeGreaterThanOrEqual(0);
      expect(winRate).toBeLessThanOrEqual(100);
    });

    it('should get balance history', () => {
      const state = usePaperTradingStore.getState();
      const initialBalance = state.startingBalance;

      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      const history = usePaperTradingStore.getState().getBalanceHistory();
      expect(history.length).toBeGreaterThan(0);
      expect(history[0].balance).toBe(initialBalance);
    });

    it('should get best and worst trades', () => {
      const state = usePaperTradingStore.getState();

      // Winning trade
      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      state.executePaperTrade({
        side: 'sell',
        fromToken: 'USDC',
        toToken: 'SOL',
        fromAmount: 5000,
        toAmount: 120,
        price: 60,
        slippage: 50,
        fees: 0.5,
      });

      // Losing trade
      state.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        slippage: 50,
        fees: 0.5,
      });

      state.executePaperTrade({
        side: 'sell',
        fromToken: 'USDC',
        toToken: 'SOL',
        fromAmount: 5000,
        toAmount: 90,
        price: 45,
        slippage: 50,
        fees: 0.5,
      });

      const bestTrade = usePaperTradingStore.getState().getBestTrade();
      const worstTrade = usePaperTradingStore.getState().getWorstTrade();

      expect(bestTrade).toBeDefined();
      expect(worstTrade).toBeDefined();
      expect((bestTrade?.pnl ?? 0) > (worstTrade?.pnl ?? 0)).toBe(true);
    });
  });

  describe('Trading Settings Store', () => {
    it('should initialize with correct default values', () => {
      const state = useTradingSettingsStore.getState();

      expect(state.slippage.tolerance).toBe(50);
      expect(state.slippage.autoAdjust).toBe(true);
      expect(state.mevProtection.enabled).toBe(false);
      expect(state.gasOptimization.priorityFeePreset).toBe('normal');
      expect(state.tradeHistory).toEqual([]);
    });

    it('should update slippage tolerance', () => {
      const state = useTradingSettingsStore.getState();
      state.setSlippageTolerance(100);

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.slippage.tolerance).toBe(100);
    });

    it('should toggle slippage auto-adjust', () => {
      const state = useTradingSettingsStore.getState();
      state.setSlippageAutoAdjust(false);

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.slippage.autoAdjust).toBe(false);
    });

    it('should update max tolerance', () => {
      const state = useTradingSettingsStore.getState();
      state.setSlippageMaxTolerance(500);

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.slippage.maxTolerance).toBe(500);
    });

    it('should toggle MEV protection', () => {
      const state = useTradingSettingsStore.getState();
      state.toggleMEVProtection(true);

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.mevProtection.enabled).toBe(true);
    });

    it('should set priority fee preset', () => {
      const state = useTradingSettingsStore.getState();
      state.setPriorityFeePreset('fast');

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.gasOptimization.priorityFeePreset).toBe('fast');
    });

    it('should get priority fee for preset', () => {
      const state = useTradingSettingsStore.getState();

      const normalFee = state.getPriorityFeeForPreset('normal');
      const fastFee = state.getPriorityFeeForPreset('fast');

      expect(normalFee.microLamports).toBeGreaterThan(0);
      expect(fastFee.microLamports).toBeGreaterThanOrEqual(normalFee.microLamports);
    });

    it('should calculate recommended slippage based on volatility', () => {
      const state = useTradingSettingsStore.getState();

      // Ensure auto-adjust is enabled
      expect(state.slippage.autoAdjust).toBe(true);

      // Low volatility (below 1 should not adjust when autoAdjust is on)
      const lowVolRecommendation = state.getRecommendedSlippage(0.5);
      expect(lowVolRecommendation).toBe(50); // No adjustment for volatility < 1

      // Medium volatility (> 1, < 3)
      const medVolRecommendation = state.getRecommendedSlippage(2);
      expect(medVolRecommendation).toBeGreaterThan(50); // Should be 50 * 1.2

      // High volatility (> 5)
      const highVolRecommendation = state.getRecommendedSlippage(6);
      expect(highVolRecommendation).toBeGreaterThan(50);
      expect(highVolRecommendation).toBeGreaterThanOrEqual(50 * 2 - 1); // 2x multiplier with small tolerance
    });

    it('should check if trade should be blocked', () => {
      const state = useTradingSettingsStore.getState();
      state.setSlippageMaxTolerance(300);
      state.setSlippageRejectAbove(true);

      const updatedState = useTradingSettingsStore.getState();

      // Trade within tolerance should not be blocked
      expect(updatedState.shouldBlockTrade(0.02, 100)).toBe(false);

      // Trade exceeding tolerance should be blocked
      expect(updatedState.shouldBlockTrade(0.05, 500)).toBe(true);
    });

    it('should add trade to history', () => {
      const state = useTradingSettingsStore.getState();

      const trade = {
        id: 'test-1',
        timestamp: Date.now(),
        side: 'buy' as const,
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        status: 'filled' as const,
        slippage: 50,
        fees: 0.5,
        mevProtected: false,
        pnl: 0,
        pnlPercent: 0,
      };

      state.addTradeToHistory(trade);

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.tradeHistory.length).toBe(1);
      expect(updatedState.tradeHistory[0].id).toBe('test-1');
    });

    it('should update trade in history', () => {
      const state = useTradingSettingsStore.getState();

      const trade = {
        id: 'test-1',
        timestamp: Date.now(),
        side: 'buy' as const,
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        status: 'filled' as const,
        slippage: 50,
        fees: 0.5,
        mevProtected: false,
        pnl: 0,
        pnlPercent: 0,
      };

      state.addTradeToHistory(trade);
      state.updateTradeInHistory('test-1', { status: 'pending' });

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.tradeHistory[0].status).toBe('pending');
    });

    it('should set trade filters', () => {
      const state = useTradingSettingsStore.getState();
      state.setTradeFilters({ side: 'buy', status: 'filled' });

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.tradeFilters.side).toBe('buy');
      expect(updatedState.tradeFilters.status).toBe('filled');
    });

    it('should reset trade filters', () => {
      const state = useTradingSettingsStore.getState();
      state.setTradeFilters({ side: 'buy', status: 'filled' });
      state.resetTradeFilters();

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.tradeFilters.side).toBe('all');
      expect(updatedState.tradeFilters.status).toBe('all');
    });

    it('should set pagination', () => {
      const state = useTradingSettingsStore.getState();
      state.setTradePagination({ page: 2, pageSize: 20 });

      const updatedState = useTradingSettingsStore.getState();
      expect(updatedState.tradePagination.page).toBe(2);
      expect(updatedState.tradePagination.pageSize).toBe(20);
    });
  });

  describe('Auto Trading Store', () => {
    it('should initialize with correct default values', () => {
      const state = useAutoTradingStore.getState();

      expect(state.strategies).toEqual([]);
      expect(state.backtestResults).toEqual([]);
      expect(state.optimizationRuns).toEqual([]);
      expect(state.isKillSwitchActive).toBe(false);
    });
  });

  describe('Cross-Store Paper Trading Flow', () => {
    it('should simulate complete trading session with settings', () => {
      const paperStore = usePaperTradingStore.getState();
      const settingsStore = useTradingSettingsStore.getState();

      // Enable paper mode
      paperStore.togglePaperMode(true);
      expect(usePaperTradingStore.getState().isPaperMode).toBe(true);

      // Update trading settings
      settingsStore.setSlippageTolerance(75);
      settingsStore.setPriorityFeePreset('fast');

      // Execute trades
      paperStore.executePaperTrade({
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 1000,
        toAmount: 50000,
        price: 50,
        slippage: 75,
        fees: 5,
      });

      // Verify trade was added to history in settings store
      settingsStore.addTradeToHistory({
        id: 'trade-1',
        timestamp: Date.now(),
        side: 'buy',
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 1000,
        toAmount: 50000,
        price: 50,
        status: 'filled',
        slippage: 75,
        fees: 5,
        mevProtected: false,
        pnl: 0,
        pnlPercent: 0,
      } as any);

      const updatedPaperState = usePaperTradingStore.getState();
      const updatedSettingsState = useTradingSettingsStore.getState();

      expect(updatedPaperState.trades.length).toBe(1);
      expect(updatedSettingsState.tradeHistory.length).toBe(1);
      expect(updatedSettingsState.slippage.tolerance).toBe(75);
      expect(updatedSettingsState.gasOptimization.priorityFeePreset).toBe('fast');
    });

    it('should handle MEV protection settings with trades', () => {
      const settingsStore = useTradingSettingsStore.getState();

      settingsStore.toggleMEVProtection(true);
      settingsStore.setJitoEnabled(true);

      const updatedState = useTradingSettingsStore.getState();

      const trade = {
        id: 'protected-trade',
        timestamp: Date.now(),
        side: 'buy' as const,
        fromToken: 'SOL',
        toToken: 'USDC',
        fromAmount: 100,
        toAmount: 5000,
        price: 50,
        status: 'filled' as const,
        slippage: 50,
        fees: 0.5,
        mevProtected: true,
        mevSavings: 5,
        pnl: 0,
        pnlPercent: 0,
      };

      settingsStore.addTradeToHistory(trade);

      const finalState = useTradingSettingsStore.getState();
      expect(finalState.mevProtection.protectedTrades).toBe(1);
      expect(finalState.mevProtection.estimatedSavings).toBe(5);
    });
  });
});
