import { describe, it, expect, beforeEach, vi } from 'vitest';
import { act, renderHook } from '@testing-library/react';
import { useTradingStore, tradingStore } from '../../src/store/tradingStore';
import type { Order, CreateOrderRequest, OrderUpdate } from '../../src/types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');

describe('Trading Store', () => {
  beforeEach(() => {
    act(() => {
      tradingStore.getState().reset();
    });
    vi.clearAllMocks();
  });

  describe('initialize', () => {
    it('should initialize trading module', async () => {
      const { result } = renderHook(() => useTradingStore());

      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await act(async () => {
        await result.current.initialize();
      });

      expect(invoke).toHaveBeenCalledWith('trading_init');
      expect(result.current.isInitialized).toBe(true);
      expect(result.current.isLoading).toBe(false);
    });

    it('should not initialize if already initialized', async () => {
      const { result } = renderHook(() => useTradingStore());

      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await act(async () => {
        await result.current.initialize();
      });

      vi.clearAllMocks();

      await act(async () => {
        await result.current.initialize();
      });

      expect(invoke).not.toHaveBeenCalled();
    });
  });

  describe('createOrder', () => {
    const mockRequest: CreateOrderRequest = {
      orderType: 'limit',
      side: 'buy',
      inputMint: 'SOL',
      outputMint: 'USDC',
      inputSymbol: 'SOL',
      outputSymbol: 'USDC',
      amount: 1.5,
      limitPrice: 100,
      slippageBps: 50,
      priorityFeeMicroLamports: 1000,
      walletAddress: 'wallet-address',
    };

    const mockOrder: Order = {
      id: 'order-1',
      orderType: 'limit',
      side: 'buy',
      status: 'pending',
      inputMint: 'SOL',
      outputMint: 'USDC',
      inputSymbol: 'SOL',
      outputSymbol: 'USDC',
      amount: 1.5,
      filledAmount: 0,
      limitPrice: 100,
      slippageBps: 50,
      priorityFeeMicroLamports: 1000,
      walletAddress: 'wallet-address',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    it('should create order with optimistic update', async () => {
      const { result } = renderHook(() => useTradingStore());

      vi.mocked(invoke).mockResolvedValueOnce(mockOrder);

      let createdOrder: Order | undefined;
      await act(async () => {
        createdOrder = await result.current.createOrder(mockRequest);
      });

      expect(invoke).toHaveBeenCalledWith('create_order', { request: mockRequest });
      expect(createdOrder).toEqual(mockOrder);
      expect(result.current.activeOrders).toContainEqual(mockOrder);
      expect(result.current.optimisticOrders.size).toBe(0);
      expect(result.current.isLoading).toBe(false);
    });

    it('should handle create order error and remove optimistic order', async () => {
      const { result } = renderHook(() => useTradingStore());
      const errorMessage = 'Insufficient balance';

      vi.mocked(invoke).mockRejectedValueOnce(new Error(errorMessage));

      await act(async () => {
        try {
          await result.current.createOrder(mockRequest);
        } catch (error) {
          expect(String(error)).toContain(errorMessage);
        }
      });

      expect(result.current.error).toContain(errorMessage);
      expect(result.current.optimisticOrders.size).toBe(0);
      expect(result.current.activeOrders).toHaveLength(0);
    });

    it('should remove optimistic order after successful API call', async () => {
      const { result } = renderHook(() => useTradingStore());

      vi.mocked(invoke).mockResolvedValueOnce(mockOrder);

      await act(async () => {
        await result.current.createOrder(mockRequest);
      });

      expect(result.current.optimisticOrders.size).toBe(0);
      expect(result.current.activeOrders).toContainEqual(mockOrder);
    });
  });

  describe('cancelOrder', () => {
    it('should cancel order with optimistic update', async () => {
      const { result } = renderHook(() => useTradingStore());
      const orderId = 'order-1';

      act(() => {
        tradingStore.setState({
          activeOrders: [
            {
              id: orderId,
              orderType: 'limit',
              side: 'buy',
              status: 'pending',
              inputMint: 'SOL',
              outputMint: 'USDC',
              inputSymbol: 'SOL',
              outputSymbol: 'USDC',
              amount: 1.5,
              filledAmount: 0,
              slippageBps: 50,
              priorityFeeMicroLamports: 1000,
              walletAddress: 'wallet-address',
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            },
          ],
        });
      });

      vi.mocked(invoke).mockResolvedValueOnce(undefined);

      await act(async () => {
        await result.current.cancelOrder(orderId);
      });

      expect(invoke).toHaveBeenCalledWith('cancel_order', { orderId });
      expect(result.current.activeOrders).toHaveLength(0);
      expect(result.current.isLoading).toBe(false);
    });

    it('should revert optimistic cancel on error', async () => {
      const { result } = renderHook(() => useTradingStore());
      const orderId = 'order-1';
      const mockOrder: Order = {
        id: orderId,
        orderType: 'limit',
        side: 'buy',
        status: 'pending',
        inputMint: 'SOL',
        outputMint: 'USDC',
        inputSymbol: 'SOL',
        outputSymbol: 'USDC',
        amount: 1.5,
        filledAmount: 0,
        slippageBps: 50,
        priorityFeeMicroLamports: 1000,
        walletAddress: 'wallet-address',
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      act(() => {
        tradingStore.setState({
          activeOrders: [mockOrder],
        });
      });

      vi.mocked(invoke)
        .mockRejectedValueOnce(new Error('Cancel failed'))
        .mockResolvedValueOnce([mockOrder]);

      await act(async () => {
        try {
          await result.current.cancelOrder(orderId);
        } catch (error) {
          expect(String(error)).toContain('Cancel failed');
        }
      });

      expect(result.current.error).toContain('Cancel failed');
    });
  });

  describe('getActiveOrders', () => {
    it('should fetch active orders', async () => {
      const { result } = renderHook(() => useTradingStore());
      const walletAddress = 'wallet-address';
      const mockOrders: Order[] = [
        {
          id: 'order-1',
          orderType: 'limit',
          side: 'buy',
          status: 'pending',
          inputMint: 'SOL',
          outputMint: 'USDC',
          inputSymbol: 'SOL',
          outputSymbol: 'USDC',
          amount: 1.5,
          filledAmount: 0,
          slippageBps: 50,
          priorityFeeMicroLamports: 1000,
          walletAddress,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        },
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockOrders);

      await act(async () => {
        await result.current.getActiveOrders(walletAddress);
      });

      expect(invoke).toHaveBeenCalledWith('get_active_orders', { walletAddress });
      expect(result.current.activeOrders).toEqual(mockOrders);
      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('order drafts', () => {
    it('should add draft', () => {
      const { result } = renderHook(() => useTradingStore());
      const request: Partial<CreateOrderRequest> = {
        orderType: 'limit',
        side: 'buy',
        amount: 1.5,
      };

      let draft;
      act(() => {
        draft = result.current.addDraft(request);
      });

      expect(result.current.drafts).toHaveLength(1);
      expect(result.current.drafts[0].request).toEqual(request);
      expect(draft).toBeDefined();
    });

    it('should update draft', () => {
      const { result } = renderHook(() => useTradingStore());
      const request: Partial<CreateOrderRequest> = {
        orderType: 'limit',
        side: 'buy',
        amount: 1.5,
      };

      let draftId: string;
      act(() => {
        const draft = result.current.addDraft(request);
        draftId = draft.id;
      });

      act(() => {
        result.current.updateDraft(draftId!, { amount: 2.0 });
      });

      expect(result.current.drafts[0].request.amount).toBe(2.0);
    });

    it('should delete draft', () => {
      const { result } = renderHook(() => useTradingStore());
      const request: Partial<CreateOrderRequest> = {
        orderType: 'limit',
        side: 'buy',
        amount: 1.5,
      };

      let draftId: string;
      act(() => {
        const draft = result.current.addDraft(request);
        draftId = draft.id;
      });

      act(() => {
        result.current.deleteDraft(draftId!);
      });

      expect(result.current.drafts).toHaveLength(0);
    });

    it('should load draft', () => {
      const { result } = renderHook(() => useTradingStore());
      const request: Partial<CreateOrderRequest> = {
        orderType: 'limit',
        side: 'buy',
        amount: 1.5,
      };

      let draftId: string;
      act(() => {
        const draft = result.current.addDraft(request);
        draftId = draft.id;
      });

      const loaded = result.current.loadDraft(draftId!);
      expect(loaded).toEqual(request);
    });
  });

  describe('handleOrderUpdate', () => {
    it('should update order status', () => {
      const { result } = renderHook(() => useTradingStore());
      const orderId = 'order-1';

      act(() => {
        tradingStore.setState({
          activeOrders: [
            {
              id: orderId,
              orderType: 'limit',
              side: 'buy',
              status: 'pending',
              inputMint: 'SOL',
              outputMint: 'USDC',
              inputSymbol: 'SOL',
              outputSymbol: 'USDC',
              amount: 1.5,
              filledAmount: 0,
              slippageBps: 50,
              priorityFeeMicroLamports: 1000,
              walletAddress: 'wallet-address',
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            },
          ],
        });
      });

      const update: OrderUpdate = {
        orderId,
        status: 'partiallyfilled',
        filledAmount: 0.5,
      };

      act(() => {
        result.current.handleOrderUpdate(update);
      });

      expect(result.current.activeOrders[0].status).toBe('partiallyfilled');
      expect(result.current.activeOrders[0].filledAmount).toBe(0.5);
    });

    it('should move filled order to history', () => {
      const { result } = renderHook(() => useTradingStore());
      const orderId = 'order-1';

      act(() => {
        tradingStore.setState({
          activeOrders: [
            {
              id: orderId,
              orderType: 'limit',
              side: 'buy',
              status: 'pending',
              inputMint: 'SOL',
              outputMint: 'USDC',
              inputSymbol: 'SOL',
              outputSymbol: 'USDC',
              amount: 1.5,
              filledAmount: 0,
              slippageBps: 50,
              priorityFeeMicroLamports: 1000,
              walletAddress: 'wallet-address',
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            },
          ],
        });
      });

      const update: OrderUpdate = {
        orderId,
        status: 'filled',
        filledAmount: 1.5,
        txSignature: 'tx-123',
      };

      act(() => {
        result.current.handleOrderUpdate(update);
      });

      expect(result.current.activeOrders).toHaveLength(0);
      expect(result.current.orderHistory).toHaveLength(1);
      expect(result.current.orderHistory[0].status).toBe('filled');
    });
  });

  describe('reset', () => {
    it('should reset store to initial state', () => {
      const { result } = renderHook(() => useTradingStore());

      act(() => {
        tradingStore.setState({
          isInitialized: true,
          activeOrders: [
            {
              id: 'order-1',
              orderType: 'limit',
              side: 'buy',
              status: 'pending',
              inputMint: 'SOL',
              outputMint: 'USDC',
              inputSymbol: 'SOL',
              outputSymbol: 'USDC',
              amount: 1.5,
              filledAmount: 0,
              slippageBps: 50,
              priorityFeeMicroLamports: 1000,
              walletAddress: 'wallet-address',
              createdAt: new Date().toISOString(),
              updatedAt: new Date().toISOString(),
            },
          ],
        });
      });

      act(() => {
        result.current.reset();
      });

      expect(result.current.isInitialized).toBe(false);
      expect(result.current.activeOrders).toEqual([]);
      expect(result.current.error).toBeNull();
    });
  });
});
