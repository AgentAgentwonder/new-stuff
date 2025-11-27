import { useEffect, useRef } from 'react';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { tradingStore } from '../store/tradingStore';
import { walletStore } from '../store/walletStore';
import { useUIStore } from '../store/uiStore';
import type { Order, OrderUpdate } from '../types';

interface OrderTriggeredEvent {
  order_id: string;
  order_type: string;
  symbol: string;
  side: string;
  trigger_price: number;
  amount: number;
}

interface TransactionUpdateEvent {
  signature: string;
  slot: number;
  timestamp: number;
  typ?: string;
  amount?: number;
  symbol?: string;
  from?: string;
  to?: string;
}

interface CopyTradeExecutionEvent {
  config_id: string;
  name: string;
  source_wallet: string;
  amount: number;
  symbol: string;
  status: string;
  tx_signature?: string;
}

interface OrderMonitoringStoppedEvent {
  message?: string;
}

/**
 * Bridge hook that connects Tauri trading/wallet event streams to frontend stores.
 * Registers listeners for order updates, transaction updates, and copy trade executions.
 * Should be mounted once at app root level.
 */
export function useTradingEventBridge() {
  const unlistenersRef = useRef<UnlistenFn[]>([]);
  const addToast = useUIStore(state => state.addToast);

  useEffect(() => {
    let mounted = true;

    const setupListeners = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        // Listen for order updates
        const unlistenOrderUpdate = await listen<Order>('order_update', event => {
          if (!mounted) return;

          const order = event.payload;
          const update: OrderUpdate = {
            orderId: order.id,
            status: order.status,
            filledAmount: order.filledAmount,
            txSignature: order.txSignature,
            errorMessage: order.errorMessage,
          };

          tradingStore.getState().handleOrderUpdate(update);

          // Refresh wallet balances after a successful fill
          if (order.status === 'filled' && order.walletAddress) {
            walletStore
              .getState()
              .fetchBalances(order.walletAddress, true)
              .catch(err => {
                console.error('Failed to refresh balances after order fill:', err);
              });
          }
        });

        // Listen for order triggered events
        const unlistenOrderTriggered = await listen<OrderTriggeredEvent>(
          'order_triggered',
          event => {
            if (!mounted) return;

            const triggered = event.payload;
            addToast({
              type: 'info',
              title: 'Order Triggered',
              message: `${triggered.side} ${triggered.amount} ${triggered.symbol} at ${triggered.trigger_price}`,
              duration: 5000,
            });
          }
        );

        // Listen for transaction updates from Helius WebSocket
        const unlistenTransactionUpdate = await listen<TransactionUpdateEvent>(
          'transaction_update',
          event => {
            if (!mounted) return;

            const tx = event.payload;

            // Refresh wallet balances for affected addresses
            const activeAccount = walletStore.getState().activeAccount;
            if (
              activeAccount &&
              (tx.from === activeAccount.publicKey || tx.to === activeAccount.publicKey)
            ) {
              walletStore
                .getState()
                .fetchBalances(activeAccount.publicKey, true)
                .catch(err => {
                  console.error('Failed to refresh balances after transaction:', err);
                });
            }

            // Show toast for significant transactions
            if (tx.amount && tx.symbol) {
              addToast({
                type: 'info',
                title: 'Transaction Detected',
                message: `${tx.typ || 'Transaction'}: ${tx.amount} ${tx.symbol}`,
                duration: 4000,
              });
            }
          }
        );

        // Listen for copy trade executions
        const unlistenCopyTradeExecution = await listen<CopyTradeExecutionEvent>(
          'copy_trade_execution',
          event => {
            if (!mounted) return;

            const execution = event.payload;
            const isSuccess = execution.status === 'success';

            addToast({
              type: isSuccess ? 'success' : 'error',
              title: 'Copy Trade Executed',
              message: `${execution.name}: ${execution.amount} ${execution.symbol}`,
              duration: 5000,
            });

            // Optionally refresh active orders after copy trade
            const activeAccount = walletStore.getState().activeAccount;
            if (activeAccount && isSuccess) {
              walletStore
                .getState()
                .fetchBalances(activeAccount.publicKey, true)
                .catch(err => {
                  console.error('Failed to refresh balances after copy trade:', err);
                });
            }
          }
        );

        // Listen for order monitoring stopped event
        const unlistenMonitoringStopped = await listen<string | OrderMonitoringStoppedEvent>(
          'order_monitoring_stopped',
          event => {
            if (!mounted) return;

            const message =
              typeof event.payload === 'string'
                ? event.payload
                : event.payload.message || 'Order monitoring stopped';

            console.error('Order monitoring stopped:', message);
            tradingStore.getState().setError('Order monitoring stopped unexpectedly');

            addToast({
              type: 'error',
              title: 'Trading Alert',
              message: 'Order monitoring stopped. Orders may not execute automatically.',
              duration: 10000,
            });
          }
        );

        // Store unlisteners for cleanup
        unlistenersRef.current = [
          unlistenOrderUpdate,
          unlistenOrderTriggered,
          unlistenTransactionUpdate,
          unlistenCopyTradeExecution,
          unlistenMonitoringStopped,
        ];

        console.log('[TradingEventBridge] Event listeners registered');
      } catch (error) {
        console.error('[TradingEventBridge] Failed to setup event listeners:', error);
        if (mounted) {
          addToast({
            type: 'error',
            title: 'Event Bridge Error',
            message: 'Failed to connect to trading events',
            duration: 8000,
          });
        }
      }
    };

    setupListeners();

    // Cleanup function
    return () => {
      mounted = false;
      console.log('[TradingEventBridge] Cleaning up event listeners');

      // Call all unlisten functions
      unlistenersRef.current.forEach(unlisten => {
        unlisten();
      });
      unlistenersRef.current = [];
    };
  }, [addToast]);
}
