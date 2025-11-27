import { useState } from 'react';
import { useSelectedCrypto, useSettingsStore } from '@/store';

// Phantom wallet supported cryptocurrencies with mock data
const PHANTOM_CRYPTOS = [
  { symbol: 'SOL', name: 'Solana', balance: 45.5, usdValue: 12340.55 },
  { symbol: 'USDC', name: 'USD Coin', balance: 5000, usdValue: 5000 },
  { symbol: 'USDT', name: 'Tether', balance: 3200, usdValue: 3200 },
  { symbol: 'ORCA', name: 'Orca', balance: 850, usdValue: 2550.75 },
  { symbol: 'RAY', name: 'Raydium', balance: 420, usdValue: 1680.32 },
  { symbol: 'COPE', name: 'Cope', balance: 500, usdValue: 125.5 },
];

export default function CryptoSelector() {
  const selectedCryptoSymbol = useSelectedCrypto();
  const updateSetting = useSettingsStore(state => state.updateSetting);
  const [isOpen, setIsOpen] = useState(false);

  const selectedCrypto =
    PHANTOM_CRYPTOS.find(c => c.symbol === selectedCryptoSymbol) || PHANTOM_CRYPTOS[0];

  const handleSelect = (symbol: string) => {
    updateSetting('selectedCrypto', symbol);
    setIsOpen(false);
  };

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted/20 transition-colors text-sm border border-border"
      >
        <div>
          <div className="font-semibold text-foreground">{selectedCrypto.symbol}</div>
          <div className="text-xs text-muted-foreground">
            ${selectedCrypto.usdValue.toLocaleString('en-US', { maximumFractionDigits: 2 })}
          </div>
        </div>
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-72 bg-card border border-border rounded-lg shadow-lg z-50">
          <div className="p-4 border-b border-border">
            <h3 className="font-semibold text-foreground text-sm">Select Cryptocurrency</h3>
          </div>
          <div className="max-h-96 overflow-y-auto">
            {PHANTOM_CRYPTOS.map(crypto => (
              <button
                key={crypto.symbol}
                onClick={() => handleSelect(crypto.symbol)}
                className={`w-full px-4 py-3 text-left hover:bg-muted/30 transition-colors border-b border-border/50 last:border-b-0 ${
                  crypto.symbol === selectedCryptoSymbol ? 'bg-accent/10' : ''
                }`}
              >
                <div className="flex justify-between items-start">
                  <div>
                    <div className="font-medium text-foreground">{crypto.symbol}</div>
                    <div className="text-xs text-muted-foreground">{crypto.name}</div>
                  </div>
                  <div className="text-right">
                    <div className="font-medium text-foreground">
                      {crypto.balance.toLocaleString()}
                    </div>
                    <div className="text-xs text-accent">
                      ${crypto.usdValue.toLocaleString('en-US', { maximumFractionDigits: 2 })}
                    </div>
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
