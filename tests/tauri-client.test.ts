/**
 * @vitest-environment jsdom
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  walletCommands,
  tradingCommands,
  aiCommands,
  portfolioCommands,
  StreamingCommandManager,
  isSuccess,
  unwrapResponse,
} from '../src/lib/tauri/commands';
import type { TokenBalance, CreateOrderRequest, ChatResponse } from '../src/lib/tauri/types';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

describe('Tauri Commands Client', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('walletCommands', () => {
    it('should handle successful getTokenBalances call', async () => {
      const mockBalances: TokenBalance[] = [
        {
          mint: 'So11111111111111111111111111111111111111112',
          symbol: 'SOL',
          name: 'Solana',
          balance: 1.5,
          decimals: 9,
          usdValue: 150.0,
          change24h: 2.5,
          logoUri: 'https://example.com/sol.png',
          lastUpdated: '2023-01-01T00:00:00Z',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockBalances);

      const result = await walletCommands.getTokenBalances('test-address', true);

      expect(invoke).toHaveBeenCalledWith('wallet_get_token_balances', {
        address: 'test-address',
        force_refresh: true,
      });
      expect(result).toEqual({
        data: mockBalances,
        success: true,
      });
    });

    it('should handle failed getTokenBalances call', async () => {
      const error = new Error('Network error');
      vi.mocked(invoke).mockRejectedValue(error);

      const result = await walletCommands.getTokenBalances('test-address');

      expect(result).toEqual({
        data: null,
        success: false,
        error: {
          message: 'Network error',
        },
      });
    });

    it('should handle estimateFee call', async () => {
      const mockFeeEstimate = {
        baseFee: 0.00001,
        priorityFee: 0.000001,
        totalFee: 0.000011,
        estimatedUnits: 200000,
      };

      vi.mocked(invoke).mockResolvedValue(mockFeeEstimate);

      const result = await walletCommands.estimateFee('recipient', 1.0, 'token-mint');

      expect(invoke).toHaveBeenCalledWith('wallet_estimate_fee', {
        recipient: 'recipient',
        amount: 1.0,
        token_mint: 'token-mint',
      });
      expect(result).toEqual({
        data: mockFeeEstimate,
        success: true,
      });
    });
  });

  describe('tradingCommands', () => {
    it('should handle createOrder call', async () => {
      const mockOrder = {
        id: 'order-123',
        orderType: 'limit',
        side: 'buy',
        status: 'pending',
        // ... other order fields
      };

      const mockRequest: CreateOrderRequest = {
        orderType: 'limit',
        side: 'buy',
        inputMint: 'input-mint',
        outputMint: 'output-mint',
        inputSymbol: 'SOL',
        outputSymbol: 'USDC',
        amount: 1.0,
        walletAddress: 'wallet-address',
      };

      vi.mocked(invoke).mockResolvedValue(mockOrder);

      const result = await tradingCommands.createOrder(mockRequest);

      expect(invoke).toHaveBeenCalledWith('create_order', {
        request: mockRequest,
      });
      expect(result).toEqual({
        data: mockOrder,
        success: true,
      });
    });

    it('should handle cancelOrder call', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const result = await tradingCommands.cancelOrder('order-123');

      expect(invoke).toHaveBeenCalledWith('cancel_order', {
        order_id: 'order-123',
      });
      expect(result).toEqual({
        data: undefined,
        success: true,
      });
    });
  });

  describe('aiCommands', () => {
    it('should handle chatMessage call', async () => {
      const mockResponse: ChatResponse = {
        content: 'AI response',
        reasoning: [
          {
            step: 1,
            description: 'Analyzing query',
            confidence: 0.95,
          },
        ],
        metadata: {
          model: 'gpt-4',
          tokens: 100,
        },
      };

      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const result = await aiCommands.chatMessage('Hello', 'general', [
        { role: 'user', content: 'Previous message' },
      ]);

      expect(invoke).toHaveBeenCalledWith('ai_chat_message', {
        message: 'Hello',
        command_type: 'general',
        history: [{ role: 'user', content: 'Previous message' }],
      });
      expect(result).toEqual({
        data: mockResponse,
        success: true,
      });
    });
  });

  describe('portfolioCommands', () => {
    it('should handle calculateAnalytics call', async () => {
      const mockPositions = [
        { symbol: 'SOL', amount: 10, value: 1500 },
        { symbol: 'USDC', amount: 1000, value: 1000 },
      ];

      const mockAnalytics = {
        correlation: {
          symbols: ['SOL', 'USDC'],
          matrix: [
            [1.0, 0.1],
            [0.1, 1.0],
          ],
          calculatedAt: '2023-01-01T00:00:00Z',
        },
        diversification: {
          score: 0.8,
          effectiveN: 1.8,
          avgCorrelation: 0.1,
          concentrationRisk: 0.2,
        },
        // ... other analytics fields
      };

      vi.mocked(invoke).mockResolvedValue(mockAnalytics);

      const result = await portfolioCommands.calculateAnalytics(mockPositions);

      expect(invoke).toHaveBeenCalledWith('calculate_portfolio_analytics', {
        positions: mockPositions,
      });
      expect(result).toEqual({
        data: mockAnalytics,
        success: true,
      });
    });
  });

  describe('StreamingCommandManager', () => {
    let mockUnlisten: UnlistenFn;

    beforeEach(() => {
      mockUnlisten = vi.fn();
      vi.mocked(listen).mockResolvedValue(mockUnlisten);
      vi.mocked(invoke).mockResolvedValue(undefined);
    });

    afterEach(() => {
      StreamingCommandManager.stopAllStreams();
    });

    it('should start chat stream successfully', async () => {
      const onChunk = vi.fn();
      const mockChunk = {
        id: 'chunk-1',
        content: 'Hello',
        finished: false,
      };

      vi.mocked(listen).mockImplementation((event, callback) => {
        // Simulate immediate chunk
        setTimeout(() => {
          callback({ payload: mockChunk });
        }, 0);
        return Promise.resolve(mockUnlisten);
      });

      const streamId = await StreamingCommandManager.startChatStream(
        'Hello',
        'general',
        [],
        onChunk
      );

      expect(invoke).toHaveBeenCalledWith('ai_chat_message_stream', {
        message: 'Hello',
        command_type: 'general',
        history: [],
        stream_id: expect.stringMatching(/^stream-\d+-[a-z0-9]+$/),
      });
      expect(listen).toHaveBeenCalledWith(
        expect.stringMatching(/^ai-chat-chunk-stream-\d+-[a-z0-9]+$/),
        expect.any(Function)
      );
      expect(streamId).toMatch(/^stream-\d+-[a-z0-9]+$/);

      // Wait for async callback
      await new Promise(resolve => setTimeout(resolve, 10));
      expect(onChunk).toHaveBeenCalledWith(mockChunk);
    });

    it('should stop stream correctly', () => {
      const streamId = 'test-stream-id';

      // Mock active listener
      StreamingCommandManager['activeListeners'].set(streamId, mockUnlisten);

      StreamingCommandManager.stopChatStream(streamId);

      expect(mockUnlisten).toHaveBeenCalled();
      expect(StreamingCommandManager['activeListeners'].has(streamId)).toBe(false);
    });

    it('should stop all streams', () => {
      const streamId1 = 'stream-1';
      const streamId2 = 'stream-2';

      StreamingCommandManager['activeListeners'].set(streamId1, mockUnlisten);
      StreamingCommandManager['activeListeners'].set(streamId2, mockUnlisten);

      StreamingCommandManager.stopAllStreams();

      expect(mockUnlisten).toHaveBeenCalledTimes(2);
      expect(StreamingCommandManager['activeListeners'].size).toBe(0);
    });
  });

  describe('Utility functions', () => {
    it('should check if response is successful', () => {
      const successResponse = { success: true, data: 'test' };
      const failureResponse = { success: false, error: { message: 'error' } };

      expect(isSuccess(successResponse)).toBe(true);
      expect(isSuccess(failureResponse)).toBe(false);
    });

    it('should unwrap successful response', () => {
      const successResponse = { success: true, data: 'test' };
      expect(unwrapResponse(successResponse)).toBe('test');
    });

    it('should throw error for failed response', () => {
      const failureResponse = {
        success: false,
        error: { message: 'Test error' },
      };

      expect(() => unwrapResponse(failureResponse)).toThrow('Test error');
    });

    it('should throw generic error for failed response without message', () => {
      const failureResponse = {
        success: false,
        error: undefined,
      };

      expect(() => unwrapResponse(failureResponse)).toThrow('Command failed');
    });
  });
});
