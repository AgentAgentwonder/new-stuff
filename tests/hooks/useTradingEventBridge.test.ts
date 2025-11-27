/**
 * @vitest-environment jsdom
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTradingEventBridge } from '../../src/hooks/useTradingEventBridge';
import { tradingStore } from '../../src/store/tradingStore';
import { walletStore } from '../../src/store/walletStore';
import type { Order } from '../../src/types';

// Mock Tauri event system
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Mock stores
vi.mock('../../src/store/tradingStore', () => ({
  tradingStore: {
    getState: vi.fn(),
  },
}));

vi.mock('../../src/store/walletStore', () => ({
  walletStore: {
    getState: vi.fn(),
  },
}));

vi.mock('../../src/store/uiStore', () => ({
  useUIStore: vi.fn(() => vi.fn()),
}));

describe('useTradingEventBridge', () => {
  const mockHandleOrderUpdate = vi.fn();
  const mockFetchBalances = vi.fn().mockResolvedValue(undefined);
  const mockSetError = vi.fn();
  const mockUnlisten = vi.fn();

  beforeEach(async () => {
    vi.clearAllMocks();

    // Setup store mocks
    vi.mocked(tradingStore.getState).mockReturnValue({
      handleOrderUpdate: mockHandleOrderUpdate,
      setError: mockSetError,
    } as any);

    vi.mocked(walletStore.getState).mockReturnValue({
      activeAccount: { publicKey: 'test-wallet', balance: 1000, network: 'mainnet' },
      fetchBalances: mockFetchBalances,
    } as any);

    // Reset the listen mock
    const { listen } = await import('@tauri-apps/api/event');
    vi.mocked(listen).mockResolvedValue(mockUnlisten);
  });

  it('should register all event listeners on mount', async () => {
    const { listen } = await import('@tauri-apps/api/event');

    renderHook(() => useTradingEventBridge());

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith('order_update', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('order_triggered', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('transaction_update', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('copy_trade_execution', expect.any(Function));
      expect(listen).toHaveBeenCalledWith('order_monitoring_stopped', expect.any(Function));
    });
  });

  it('should handle order_update event', async () => {
    const { listen } = await import('@tauri-apps/api/event');
    let orderUpdateHandler: any;

    vi.mocked(listen).mockImplementation((eventName: string, handler: any) => {
      if (eventName === 'order_update') {
        orderUpdateHandler = handler;
      }
      return Promise.resolve(mockUnlisten);
    });

    renderHook(() => useTradingEventBridge());

    await waitFor(() => {
      expect(orderUpdateHandler).toBeDefined();
    });

    // Simulate order_update event
    const mockOrder: Order = {
      id: 'order-123',
      orderType: 'market',
      side: 'buy',
      status: 'filled',
      inputMint: 'SOL',
      outputMint: 'USDC',
      inputSymbol: 'SOL',
      outputSymbol: 'USDC',
      amount: 10,
      filledAmount: 10,
      slippageBps: 100,
      priorityFeeMicroLamports: 1000,
      walletAddress: 'test-wallet',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      txSignature: 'tx-123',
    };

    orderUpdateHandler({ payload: mockOrder });

    await waitFor(() => {
      expect(mockHandleOrderUpdate).toHaveBeenCalledWith({
        orderId: 'order-123',
        status: 'filled',
        filledAmount: 10,
        txSignature: 'tx-123',
        errorMessage: undefined,
      });
      expect(mockFetchBalances).toHaveBeenCalledWith('test-wallet', true);
    });
  });

  it('should cleanup listeners on unmount', async () => {
    const { unmount } = renderHook(() => useTradingEventBridge());

    await waitFor(() => {
      expect(mockUnlisten).not.toHaveBeenCalled();
    });

    unmount();

    await waitFor(() => {
      expect(mockUnlisten).toHaveBeenCalledTimes(5);
    });
  });

  it('should not process events after unmount', async () => {
    const { listen } = await import('@tauri-apps/api/event');
    let orderUpdateHandler: any;

    vi.mocked(listen).mockImplementation((eventName: string, handler: any) => {
      if (eventName === 'order_update') {
        orderUpdateHandler = handler;
      }
      return Promise.resolve(mockUnlisten);
    });

    const { unmount } = renderHook(() => useTradingEventBridge());

    await waitFor(() => {
      expect(orderUpdateHandler).toBeDefined();
    });

    unmount();

    // Try to trigger event after unmount
    const mockOrder: Order = {
      id: 'order-999',
      orderType: 'market',
      side: 'sell',
      status: 'filled',
      inputMint: 'USDC',
      outputMint: 'SOL',
      inputSymbol: 'USDC',
      outputSymbol: 'SOL',
      amount: 100,
      filledAmount: 100,
      slippageBps: 100,
      priorityFeeMicroLamports: 1000,
      walletAddress: 'test-wallet',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    mockHandleOrderUpdate.mockClear();
    orderUpdateHandler({ payload: mockOrder });

    // Event should not be processed after unmount
    await new Promise(resolve => setTimeout(resolve, 100));
    expect(mockHandleOrderUpdate).not.toHaveBeenCalled();
  });
});
