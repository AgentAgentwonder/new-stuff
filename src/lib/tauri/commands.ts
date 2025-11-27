import type { UnlistenFn } from '@tauri-apps/api/event';
import type {
  TokenBalance,
  SendTransactionInput,
  TransactionFeeEstimate,
  QRCodeData,
  SolanaPayQR,
  CreateOrderRequest,
  Order,
  ChatMessage,
  ChatResponse,
  PortfolioOptimization,
  PatternWarning,
  PortfolioAnalytics,
  SectorAllocation,
  TauriError,
  ApiResponse,
  StreamingChunk,
} from './types';

// Error handling utility
const handleTauriError = (error: any): TauriError => {
  if (typeof error === 'string') {
    return { message: error };
  }
  if (error?.message) {
    return {
      message: error.message,
      code: error.code,
      details: error.details,
    };
  }
  return { message: 'Unknown error occurred' };
};

// Generic wrapper for Tauri commands
const wrapCommand = async <T>(
  command: string,
  args?: Record<string, any>
): Promise<ApiResponse<T>> => {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke<T>(command, args);
    return {
      data: result,
      success: true,
    };
  } catch (error) {
    return {
      data: null as any,
      success: false,
      error: handleTauriError(error),
    };
  }
};

// Wallet Commands
export const walletCommands = {
  getTokenBalances: (address: string, forceRefresh = false) =>
    wrapCommand<TokenBalance[]>('wallet_get_token_balances', {
      address,
      force_refresh: forceRefresh,
    }),

  estimateFee: (recipient: string, amount: number, tokenMint?: string) =>
    wrapCommand<TransactionFeeEstimate>('wallet_estimate_fee', {
      recipient,
      amount,
      token_mint: tokenMint,
    }),

  sendTransaction: (input: SendTransactionInput, walletAddress: string) =>
    wrapCommand<string>('wallet_send_transaction', {
      input,
      wallet_address: walletAddress,
    }),

  generateQR: (data: QRCodeData) => wrapCommand<string>('wallet_generate_qr', { data }),

  generateSolanaPayQR: (
    recipient: string,
    amount?: number,
    splToken?: string,
    reference?: string,
    label?: string,
    message?: string,
    memo?: string
  ) =>
    wrapCommand<SolanaPayQR>('wallet_generate_solana_pay_qr', {
      recipient,
      amount,
      spl_token: splToken,
      reference,
      label,
      message,
      memo,
    }),
};

// Trading Commands
export const tradingCommands = {
  init: () => wrapCommand<void>('trading_init'),

  createOrder: (request: CreateOrderRequest) => wrapCommand<Order>('create_order', { request }),

  cancelOrder: (orderId: string) => wrapCommand<void>('cancel_order', { order_id: orderId }),

  getActiveOrders: (walletAddress: string) =>
    wrapCommand<Order[]>('get_active_orders', {
      wallet_address: walletAddress,
    }),

  getOrderHistory: (walletAddress: string, limit?: number) =>
    wrapCommand<Order[]>('get_order_history', {
      wallet_address: walletAddress,
      limit,
    }),

  getOrder: (orderId: string) => wrapCommand<Order>('get_order', { order_id: orderId }),
};

// AI Commands
export const aiCommands = {
  chatMessage: (message: string, commandType?: string, history?: ChatMessage[]) =>
    wrapCommand<ChatResponse>('ai_chat_message', {
      message,
      command_type: commandType,
      history: history || [],
    }),

  getPatternWarnings: () => wrapCommand<PatternWarning[]>('ai_get_pattern_warnings'),

  optimizePortfolio: (currentAllocation: Record<string, number>, riskTolerance?: number) =>
    wrapCommand<PortfolioOptimization>('ai_optimize_portfolio', {
      current_allocation: currentAllocation,
      risk_tolerance: riskTolerance,
    }),
};

// Portfolio Analytics Commands
export const portfolioCommands = {
  calculateAnalytics: (positions: Array<{ symbol: string; amount: number; value: number }>) =>
    wrapCommand<PortfolioAnalytics>('calculate_portfolio_analytics', {
      positions,
    }),

  getSectorAllocation: (positions: Array<{ symbol: string; amount: number; value: number }>) =>
    wrapCommand<SectorAllocation[]>('get_sector_allocation', {
      positions,
    }),

  clearCache: () => wrapCommand<void>('clear_portfolio_cache'),
};

// Streaming Commands (for AI chat)
export class StreamingCommandManager {
  private static activeListeners = new Map<string, UnlistenFn>();

  static async startChatStream(
    message: string,
    commandType?: string,
    history?: ChatMessage[],
    onChunk: (chunk: StreamingChunk) => void
  ): Promise<string> {
    const streamId = `stream-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');

      // Start the stream
      await invoke('ai_chat_message_stream', {
        message,
        command_type: commandType,
        history: history || [],
        stream_id: streamId,
      });

      // Listen for chunks
      const unlisten = await listen<StreamingChunk>(`ai-chat-chunk-${streamId}`, event => {
        onChunk(event.payload);

        // Clean up listener when stream is finished
        if (event.payload.finished || event.payload.error) {
          this.stopChatStream(streamId);
        }
      });

      this.activeListeners.set(streamId, unlisten);
      return streamId;
    } catch (error) {
      // Clean up on error
      this.stopChatStream(streamId);
      throw error;
    }
  }

  static stopChatStream(streamId: string): void {
    const unlisten = this.activeListeners.get(streamId);
    if (unlisten) {
      unlisten();
      this.activeListeners.delete(streamId);
    }
  }

  static stopAllStreams(): void {
    for (const [streamId, unlisten] of this.activeListeners) {
      unlisten();
    }
    this.activeListeners.clear();
  }
}

// Utility function to check if a response is successful
export const isSuccess = <T>(
  response: ApiResponse<T>
): response is ApiResponse<T> & { data: T } => {
  return response.success;
};

// Utility function to extract data or throw error
export const unwrapResponse = <T>(response: ApiResponse<T>): T => {
  if (!response.success) {
    throw new Error(response.error?.message || 'Command failed');
  }
  return response.data;
};
