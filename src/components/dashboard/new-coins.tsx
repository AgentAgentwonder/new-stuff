import { useState, useEffect } from 'react';
import { useSettingsStore, useShallow } from '@/store';
import { Card } from '@/components/ui/card';

interface Coin {
  symbol: string;
  name: string;
  marketCap: number;
  athMarketCap: number;
  price: number;
  holders: number;
  totalFees: number;
  change24h: number;
  createdAt: Date;
  riskLevel: 'low' | 'medium' | 'high';
}

const generateMockCoins = (): Coin[] => [
  {
    symbol: 'BONK',
    name: 'Bonk',
    marketCap: 850000000,
    athMarketCap: 950000000,
    price: 0.000045,
    holders: 125000,
    totalFees: 15000,
    change24h: 12.5,
    createdAt: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
    riskLevel: 'high',
  },
  {
    symbol: 'JUP',
    name: 'Jupiter',
    marketCap: 450000000,
    athMarketCap: 520000000,
    price: 0.85,
    holders: 89000,
    totalFees: 12000,
    change24h: 8.3,
    createdAt: new Date(Date.now() - 14 * 24 * 60 * 60 * 1000),
    riskLevel: 'medium',
  },
  {
    symbol: 'PYTH',
    name: 'Pyth Network',
    marketCap: 320000000,
    athMarketCap: 400000000,
    price: 0.62,
    holders: 67000,
    totalFees: 8500,
    change24h: -2.1,
    createdAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000),
    riskLevel: 'medium',
  },
  {
    symbol: 'DRIFT',
    name: 'Drift Protocol',
    marketCap: 180000000,
    athMarketCap: 250000000,
    price: 3.2,
    holders: 45000,
    totalFees: 5000,
    change24h: 15.7,
    createdAt: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000),
    riskLevel: 'high',
  },
];

export default function NewCoins() {
  const {
    minMarketCap,
    buyInAmounts,
    defaultBuyInAmount,
    updateSetting,
    addBuyInPreset,
    removeBuyInPreset,
  } = useSettingsStore(
    useShallow(state => ({
      minMarketCap: state.minMarketCap,
      buyInAmounts: state.buyInAmounts,
      defaultBuyInAmount: state.defaultBuyInAmount,
      updateSetting: state.updateSetting,
      addBuyInPreset: state.addBuyInPreset,
      removeBuyInPreset: state.removeBuyInPreset,
    }))
  );
  const [coins, setCoins] = useState<Coin[]>([]);
  const [sortBy, setSortBy] = useState<'marketCap' | 'price' | 'holders' | 'athMarketCap' | 'age'>(
    'marketCap'
  );
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [customBuyIn, setCustomBuyIn] = useState('');
  const [mounted, setMounted] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);

  useEffect(() => {
    setMounted(true);
    setCoins(generateMockCoins());
    const interval = setInterval(() => {
      setIsUpdating(true);
      setCoins(generateMockCoins());
      setTimeout(() => setIsUpdating(false), 200);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  const getCoinAge = (createdAt: Date) => {
    const now = new Date();
    const diffMs = now.getTime() - createdAt.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const diffHours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const diffMins = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    if (diffDays > 0) return `${diffDays}d ${diffHours}h`;
    if (diffHours > 0) return `${diffHours}h ${diffMins}m`;
    return `${diffMins}m`;
  };

  const sortedCoins = [...coins]
    .filter(coin => coin.marketCap >= minMarketCap)
    .sort((a, b) => {
      let aVal, bVal;
      switch (sortBy) {
        case 'marketCap':
          aVal = a.marketCap;
          bVal = b.marketCap;
          break;
        case 'athMarketCap':
          aVal = a.athMarketCap;
          bVal = b.athMarketCap;
          break;
        case 'price':
          aVal = a.price;
          bVal = b.price;
          break;
        case 'holders':
          aVal = a.holders;
          bVal = b.holders;
          break;
        case 'age':
          aVal = a.createdAt.getTime();
          bVal = b.createdAt.getTime();
          break;
        default:
          return 0;
      }
      return sortOrder === 'desc' ? bVal - aVal : aVal - bVal;
    });

  const handleAddCustomBuyIn = () => {
    const amount = Number.parseFloat(customBuyIn);
    if (!isNaN(amount) && amount > 0) {
      addBuyInPreset(amount);
      setCustomBuyIn('');
    }
  };

  const handleRemoveBuyIn = (amount: number) => {
    removeBuyInPreset(amount);
  };

  const riskIndicator = (risk: string) => {
    const colors = {
      low: 'bg-green-500/20 text-green-400 border-green-500/50',
      medium: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50',
      high: 'bg-red-500/20 text-red-400 border-red-500/50',
    };
    return colors[risk as keyof typeof colors] || colors.medium;
  };

  const getCryptoBuyAmount = (usdAmount: number, price: number, symbol: string) => {
    const cryptoAmount = usdAmount / price;
    return `${cryptoAmount.toFixed(4)} ${symbol} ($${usdAmount})`;
  };

  if (!mounted) {
    return (
      <Card className="bg-card border-border p-4 h-full">
        <div className="animate-pulse space-y-2">
          <div className="h-3 bg-muted rounded w-1/3"></div>
          <div className="h-20 bg-muted rounded"></div>
        </div>
      </Card>
    );
  }

  return (
    <Card className="bg-card border-border p-3 h-full flex flex-col">
      <div className="flex-1 flex flex-col overflow-hidden space-y-2">
        {/* Header with Live Indicator and Sort Buttons */}
        <div className="flex items-center justify-between gap-2 mb-1">
          <div className="flex items-center gap-1 flex-1 min-w-0">
            <h2 className="text-base font-semibold text-foreground whitespace-nowrap">
              📈 New Coins
            </h2>
            <div className="flex items-center gap-1">
              <div
                className={`w-2 h-2 rounded-full ${isUpdating ? 'bg-accent animate-pulse' : 'bg-accent/50'}`}
              />
              <span className="text-xs text-muted-foreground">Live</span>
            </div>
          </div>
          <div className="flex gap-0.5 overflow-x-auto">
            {(['marketCap', 'athMarketCap', 'price', 'holders', 'age'] as const).map(metric => (
              <button
                key={metric}
                onClick={() => {
                  if (sortBy === metric) {
                    setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
                  } else {
                    setSortBy(metric);
                    setSortOrder('desc');
                  }
                }}
                className={`flex items-center gap-0.5 text-xs px-1.5 py-0.5 rounded border transition-colors whitespace-nowrap ${
                  sortBy === metric
                    ? 'bg-accent border-accent text-primary-foreground'
                    : 'border-border hover:border-accent hover:text-accent'
                }`}
              >
                <span className="text-xs">
                  {metric === 'marketCap' && 'MC'}
                  {metric === 'athMarketCap' && 'ATH'}
                  {metric === 'price' && 'Price'}
                  {metric === 'holders' && 'Hold'}
                  {metric === 'age' && 'Age'}
                </span>
                {sortBy === metric && (sortOrder === 'asc' ? '↑' : '↓')}
              </button>
            ))}
          </div>
        </div>

        {/* Metric Labels */}
        <div className="grid grid-cols-6 gap-1 px-1 py-0.5 text-xs text-muted-foreground font-medium">
          <div className="col-span-1">Symbol</div>
          <div>MC</div>
          <div>ATH</div>
          <div>Price</div>
          <div>Holders</div>
          <div className="text-right">Fees</div>
        </div>

        {/* Main Layout: Coins on Left, Controls on Right */}
        <div className="flex-1 flex gap-2 overflow-hidden min-h-0">
          {/* Left: Coin List */}
          <div className="flex-1 flex flex-col overflow-hidden">
            <div className="flex-1 overflow-y-auto space-y-0.5 pr-1">
              {sortedCoins.slice(0, 5).map(coin => (
                <div
                  key={coin.symbol}
                  onClick={() => {
                    window.location.href = `/coin/${coin.symbol}`;
                  }}
                  className="border border-border rounded p-1 hover:bg-muted/5 hover:cursor-pointer transition-colors text-xs"
                >
                  <div className="grid grid-cols-6 gap-1 items-center">
                    <div className="col-span-1">
                      <div className="font-semibold text-foreground text-xs">{coin.symbol}</div>
                      <div className="text-muted-foreground text-xs">
                        Age: {getCoinAge(coin.createdAt)}
                      </div>
                    </div>
                    <div className="bg-muted/5 p-0.5 rounded text-xs">
                      <div className="text-foreground font-medium">
                        ${(coin.marketCap / 1000000).toFixed(0)}M
                      </div>
                    </div>
                    <div className="bg-muted/5 p-0.5 rounded text-xs">
                      <div className="text-foreground font-medium">
                        ${(coin.athMarketCap / 1000000).toFixed(0)}M
                      </div>
                    </div>
                    <div className="bg-muted/5 p-0.5 rounded text-xs">
                      <div className="text-foreground font-medium">${coin.price.toFixed(6)}</div>
                    </div>
                    <div className="bg-muted/5 p-0.5 rounded text-xs">
                      <div className="text-foreground font-medium">
                        {(coin.holders / 1000).toFixed(0)}k
                      </div>
                    </div>
                    <div className="flex items-center justify-between gap-0.5">
                      <div className="bg-muted/5 p-0.5 rounded text-xs flex-1">
                        <div className="text-foreground font-medium">
                          ${(coin.totalFees / 1000).toFixed(1)}k
                        </div>
                      </div>
                      <button className="text-xs px-1.5 py-0.5 rounded bg-accent hover:bg-accent/90 text-primary-foreground transition-colors font-medium whitespace-nowrap">
                        Buy
                      </button>
                    </div>
                  </div>
                  <div className="flex items-center justify-between gap-1 mt-0.5">
                    <div
                      className={`text-xs px-1 py-0.5 rounded border flex items-center gap-0.5 ${riskIndicator(coin.riskLevel)}`}
                    >
                      📊 {coin.riskLevel.toUpperCase()}
                    </div>
                    <div
                      className={
                        coin.change24h >= 0
                          ? 'text-accent text-xs font-medium'
                          : 'text-red-500 text-xs font-medium'
                      }
                    >
                      {coin.change24h >= 0 ? '+' : ''}
                      {coin.change24h}%
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Right: Controls */}
          <div className="w-40 flex flex-col gap-1.5 overflow-y-auto">
            {/* Market Cap Filter */}
            <div className="p-2 bg-muted/5 rounded border border-border">
              <label className="text-xs font-medium text-muted-foreground block mb-1">
                Min MC (M)
              </label>
              <input
                type="number"
                value={minMarketCap / 1000000}
                onChange={e =>
                  updateSetting(
                    'minMarketCap',
                    Math.max(1, Number.parseFloat(e.target.value) || 25) * 1000000
                  )
                }
                className="w-full px-2 py-1 bg-background border border-border rounded text-xs text-foreground"
              />
            </div>

            {/* Quick Buy Options */}
            <div className="p-2 bg-muted/5 rounded border border-border">
              <label className="text-xs font-medium text-muted-foreground block mb-1">
                Quick Buy ($)
              </label>
              <div className="flex flex-col gap-1 mb-1">
                <div className="flex gap-1">
                  <input
                    type="number"
                    value={customBuyIn}
                    onChange={e => setCustomBuyIn(e.target.value)}
                    placeholder="Add"
                    className="py-1 bg-background border border-border rounded text-xs text-foreground px-0.5 w-[100px]"
                  />
                  <button
                    onClick={handleAddCustomBuyIn}
                    className="px-2 py-1 bg-accent hover:bg-accent/90 text-primary-foreground rounded text-xs font-medium transition-colors"
                  >
                    Add
                  </button>
                </div>
              </div>
              <div className="flex gap-1 flex-wrap">
                {buyInAmounts.map(amount => (
                  <button
                    key={amount}
                    onClick={() => updateSetting('defaultBuyInAmount', amount)}
                    className={`text-xs px-2 py-0.5 rounded border transition-colors ${
                      defaultBuyInAmount === amount
                        ? 'bg-accent border-accent text-primary-foreground'
                        : 'border-border hover:border-accent hover:text-accent'
                    }`}
                    onContextMenu={e => {
                      e.preventDefault();
                      handleRemoveBuyIn(amount);
                    }}
                  >
                    {sortedCoins.length > 0
                      ? getCryptoBuyAmount(amount, sortedCoins[0].price, sortedCoins[0].symbol)
                      : `${amount}`}
                  </button>
                ))}
                <button className="text-xs px-2 py-0.5 rounded border border-border hover:border-accent hover:text-accent font-semibold">
                  All In
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Card>
  );
}
