import type { Order, CreateOrderRequest, OrderUpdate, QuickTradeRequest } from '../types';
import { createBoundStore } from './createBoundStore';

export interface OrderDraft {
  id: string;
  request: Partial<CreateOrderRequest>;
  createdAt: string;
}

interface TradingStoreState {
  isInitialized: boolean;
  activeOrders: Order[];
  orderHistory: Order[];
  drafts: OrderDraft[];
  optimisticOrders: Map<string, Order>;
  isLoading: boolean;
  error: string | null;

  // Actions
  initialize: () => Promise<void>;
  createOrder: (request: CreateOrderRequest) => Promise<Order>;
  cancelOrder: (orderId: string) => Promise<void>;
  getActiveOrders: (walletAddress: string) => Promise<void>;
  getOrderHistory: (walletAddress: string, limit?: number) => Promise<void>;
  getOrder: (orderId: string) => Promise<Order>;
  acknowledgeOrder: (orderId: string) => Promise<void>;
  quickTrade: (request: QuickTradeRequest) => Promise<Order>;
  addDraft: (request: Partial<CreateOrderRequest>) => OrderDraft;
  updateDraft: (id: string, request: Partial<CreateOrderRequest>) => void;
  deleteDraft: (id: string) => void;
  loadDraft: (id: string) => Partial<CreateOrderRequest> | null;
  addOptimisticOrder: (order: Order) => void;
  removeOptimisticOrder: (orderId: string) => void;
  updateOrderOptimistic: (orderId: string, update: Partial<Order>) => void;
  handleOrderUpdate: (update: OrderUpdate) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  isInitialized: false,
  activeOrders: [],
  orderHistory: [],
  drafts: [],
  optimisticOrders: new Map(),
  isLoading: false,
  error: null,
};

const storeResult = createBoundStore<TradingStoreState>((set, get) => ({
  ...initialState,

  initialize: async () => {
    if (get().isInitialized) return;

    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('trading_init');
      set({ isInitialized: true, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  createOrder: async (request: CreateOrderRequest) => {
    set({ isLoading: true, error: null });

    const optimisticOrder: Order = {
      id: `optimistic-${Date.now()}`,
      orderType: request.orderType,
      side: request.side,
      status: 'pending',
      inputMint: request.inputMint,
      outputMint: request.outputMint,
      inputSymbol: request.inputSymbol,
      outputSymbol: request.outputSymbol,
      amount: request.amount,
      filledAmount: 0,
      limitPrice: request.limitPrice,
      stopPrice: request.stopPrice,
      trailingPercent: request.trailingPercent,
      linkedOrderId: request.linkedOrderId,
      slippageBps: request.slippageBps,
      priorityFeeMicroLamports: request.priorityFeeMicroLamports,
      walletAddress: request.walletAddress,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    get().addOptimisticOrder(optimisticOrder);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const order = await invoke<Order>('create_order', { request });

      get().removeOptimisticOrder(optimisticOrder.id);

      set(state => ({
        activeOrders: [...state.activeOrders, order],
        isLoading: false,
      }));

      return order;
    } catch (error) {
      get().removeOptimisticOrder(optimisticOrder.id);
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  cancelOrder: async (orderId: string) => {
    set({ isLoading: true, error: null });

    get().updateOrderOptimistic(orderId, { status: 'cancelled' });

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('cancel_order', { orderId });
      set(state => ({
        activeOrders: state.activeOrders.filter(o => o.id !== orderId),
        isLoading: false,
      }));
    } catch (error) {
      await get().getActiveOrders(get().activeOrders[0]?.walletAddress);
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  getActiveOrders: async (walletAddress: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const orders = await invoke<Order[]>('get_active_orders', { walletAddress });
      set({ activeOrders: orders, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  getOrderHistory: async (walletAddress: string, limit = 100) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const orders = await invoke<Order[]>('get_order_history', { walletAddress, limit });
      set({ orderHistory: orders, isLoading: false });
    } catch (error) {
      set({ error: String(error), isLoading: false });
    }
  },

  getOrder: async (orderId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const order = await invoke<Order>('get_order', { orderId });
      set({ isLoading: false });
      return order;
    } catch (error) {
      set({ error: String(error), isLoading: false });
      throw error;
    }
  },

  acknowledgeOrder: async (orderId: string) => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('acknowledge_order', { orderId });
      get().updateOrderOptimistic(orderId, { status: 'pending' });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  quickTrade: async (request: QuickTradeRequest) => {
    const orderRequest: CreateOrderRequest = {
      orderType: 'market',
      side: request.side,
      inputMint: request.inputMint,
      outputMint: request.outputMint,
      inputSymbol: request.inputSymbol,
      outputSymbol: request.outputSymbol,
      amount: request.amount,
      slippageBps: 100,
      priorityFeeMicroLamports: 1000,
      walletAddress: request.walletAddress,
    };

    return get().createOrder(orderRequest);
  },

  addDraft: (request: Partial<CreateOrderRequest>) => {
    const draft: OrderDraft = {
      id: `draft-${Date.now()}`,
      request,
      createdAt: new Date().toISOString(),
    };
    set(state => ({
      drafts: [...state.drafts, draft],
    }));
    return draft;
  },

  updateDraft: (id: string, request: Partial<CreateOrderRequest>) => {
    set(state => ({
      drafts: state.drafts.map(d =>
        d.id === id ? { ...d, request: { ...d.request, ...request } } : d
      ),
    }));
  },

  deleteDraft: (id: string) => {
    set(state => ({
      drafts: state.drafts.filter(d => d.id !== id),
    }));
  },

  loadDraft: (id: string) => {
    const draft = get().drafts.find(d => d.id === id);
    return draft?.request || null;
  },

  addOptimisticOrder: (order: Order) => {
    set(state => {
      const newMap = new Map(state.optimisticOrders);
      newMap.set(order.id, order);
      return { optimisticOrders: newMap };
    });
  },

  removeOptimisticOrder: (orderId: string) => {
    set(state => {
      const newMap = new Map(state.optimisticOrders);
      newMap.delete(orderId);
      return { optimisticOrders: newMap };
    });
  },

  updateOrderOptimistic: (orderId: string, update: Partial<Order>) => {
    set(state => ({
      activeOrders: state.activeOrders.map(o => (o.id === orderId ? { ...o, ...update } : o)),
    }));
  },

  handleOrderUpdate: (update: OrderUpdate) => {
    const state = get();
    const order = state.activeOrders.find(o => o.id === update.orderId);

    if (!order) return;

    const updatedOrder: Order = {
      ...order,
      status: update.status,
      filledAmount: update.filledAmount ?? order.filledAmount,
      txSignature: update.txSignature ?? order.txSignature,
      errorMessage: update.errorMessage ?? order.errorMessage,
      updatedAt: new Date().toISOString(),
    };

    if (update.status === 'filled' || update.status === 'cancelled' || update.status === 'failed') {
      set(state => ({
        activeOrders: state.activeOrders.filter(o => o.id !== update.orderId),
        orderHistory: [updatedOrder, ...state.orderHistory],
      }));
    } else {
      get().updateOrderOptimistic(update.orderId, updatedOrder);
    }
  },

  setError: (error: string | null) => {
    if (get().error === error) return;
    set({ error });
  },

  reset: () => {
    set(initialState);
  },
}));

export const useTradingStore = storeResult.useStore;
export const tradingStore = storeResult.store;

export const useActiveOrders = () => {
  return useTradingStore(state => state.activeOrders);
};

export const useOrderDrafts = () => {
  return useTradingStore(state => state.drafts);
};

export const useCombinedOrders = () => {
  return useTradingStore(state => {
    const optimistic = Array.from(state.optimisticOrders.values());
    return [...optimistic, ...state.activeOrders];
  });
};
