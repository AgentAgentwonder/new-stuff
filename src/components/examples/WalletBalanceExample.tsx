import React from 'react';
import { useTauriCommand } from '../../hooks';
import { walletCommands } from '../../lib/tauri/commands';
import { Spinner } from '../LoadingOverlay';
import { useToast } from '../Toast';

/**
 * Example component demonstrating proper usage of Tauri client and hooks
 */
export function WalletBalanceExample() {
  const [address, setAddress] = React.useState('demo-wallet-address');
  const { success, error } = useToast();

  // Use the useTauriCommand hook for consistent loading/error handling
  const {
    data: balances,
    isLoading,
    error: balanceError,
    execute: loadBalances,
  } = useTauriCommand(
    (addr: string, forceRefresh = false) => walletCommands.getTokenBalances(addr, forceRefresh),
    {
      onSuccess: data => {
        success('Balances Loaded', `Found ${data.length} tokens`);
      },
      onError: error => {
        console.error('Failed to load balances:', error);
      },
      showToastOnError: false, // We handle errors manually
      loadingId: 'wallet-balances',
      loadingMessage: 'Loading wallet balances...',
    }
  );

  const handleRefresh = () => {
    loadBalances(address, true);
  };

  const handleAddressChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setAddress(e.target.value);
  };

  React.useEffect(() => {
    // Load balances on mount and when address changes
    loadBalances(address);
  }, [address, loadBalances]);

  return (
    <div className="p-6 bg-background-secondary rounded-lg border border-border">
      <div className="mb-6">
        <h2 className="text-xl font-semibold text-text mb-4">Wallet Balances</h2>

        <div className="flex gap-2 mb-4">
          <input
            type="text"
            value={address}
            onChange={handleAddressChange}
            placeholder="Enter wallet address"
            className="flex-1 px-3 py-2 bg-background border border-border rounded text-text"
          />
          <button
            onClick={handleRefresh}
            disabled={isLoading}
            className="px-4 py-2 bg-primary text-white rounded hover:bg-primary-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? <Spinner size="sm" /> : 'Refresh'}
          </button>
        </div>

        {balanceError && (
          <div className="p-3 bg-error/10 border border-error/20 rounded text-error text-sm">
            Error: {balanceError.message}
          </div>
        )}
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-8">
          <Spinner />
          <span className="ml-2 text-text-secondary">Loading balances...</span>
        </div>
      ) : balances && balances.length > 0 ? (
        <div className="space-y-3">
          {balances.map(balance => (
            <div
              key={balance.mint}
              className="flex items-center justify-between p-4 bg-background border border-border rounded"
            >
              <div className="flex items-center gap-3">
                {balance.logoUri && (
                  <img
                    src={balance.logoUri}
                    alt={balance.symbol}
                    className="w-8 h-8 rounded-full"
                  />
                )}
                <div>
                  <div className="font-medium text-text">{balance.symbol}</div>
                  <div className="text-sm text-text-secondary">{balance.name}</div>
                </div>
              </div>

              <div className="text-right">
                <div className="font-medium text-text">
                  {balance.balance.toFixed(balance.decimals)}
                </div>
                <div className="text-sm text-text-secondary">${balance.usdValue.toFixed(2)}</div>
                <div
                  className={`text-xs ${balance.change24h >= 0 ? 'text-success' : 'text-error'}`}
                >
                  {balance.change24h >= 0 ? '+' : ''}
                  {balance.change24h.toFixed(2)}%
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-8 text-text-secondary">
          No balances found for this address
        </div>
      )}
    </div>
  );
}
