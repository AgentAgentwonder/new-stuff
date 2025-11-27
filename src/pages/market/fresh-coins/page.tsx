import { useState, useEffect } from 'react';
import { useSettingsStore, useShallow } from '@/store';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ChevronUp, ChevronDown, TrendingUp } from 'lucide-react';

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
  {
    symbol: 'MARINADE',
    name: 'Marinade',
    marketCap: 150000000,
    athMarketCap: 180000000,
    price: 12.5,
    holders: 32000,
    totalFees: 3500,
    change24h: 5.2,
    createdAt: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000),
    riskLevel: 'low',
  },
];

export default function FreshCoinsPage() {
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
  const [selectedBuyin, setSelectedBuyin] = useState<{ [key: string]: number }>({});
  const [sortBy, setSortBy] = useState<'marketCap' | 'price' | 'holders' | 'athMarketCap' | 'age'>(
    'marketCap'
  );
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');
  const [customBuyIn, setCustomBuyIn] = useState('');
  const [showAllIn, setShowAllIn] = useState(false);

  useEffect(() => {
    setCoins(generateMockCoins());
    const interval = setInterval(() => {
      setCoins(generateMockCoins());
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

  const handleBuyIn = (coinSymbol: string, amount: number) => {
    setSelectedBuyin(prev => ({
      ...prev,
      [coinSymbol]: amount,
    }));
    console.log(`[v0] Buying in ${coinSymbol} with $${amount}`);
  };

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

  return (
    <div className="p-6 space-y-6 fade-in">
      <div>
        <h1 className="text-3xl font-bold text-foreground">Fresh Coins</h1>
        <p className="text-muted-foreground mt-1">
          Recently listed and launched tokens - updated every second
        </p>
      </div>

      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle>New Listings (Market Cap {(minMarketCap / 1000000).toFixed(0)}M+)</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 bg-muted/5 rounded-lg border border-border">
            <div>
              <label className="text-xs font-medium text-muted-foreground mb-2 block">
                Min Market Cap (M)
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
                className="w-full px-3 py-2 bg-background border border-border rounded text-sm text-foreground"
              />
            </div>
            <div>
              <label className="text-xs font-medium text-muted-foreground mb-2 block">
                Quick Buy Amount
              </label>
              <div className="flex gap-2">
                <input
                  type="number"
                  value={customBuyIn}
                  onChange={e => setCustomBuyIn(e.target.value)}
                  placeholder="Add amount"
                  className="flex-1 px-3 py-2 bg-background border border-border rounded text-sm text-foreground"
                />
                <button
                  onClick={handleAddCustomBuyIn}
                  className="px-3 py-2 bg-accent hover:bg-accent/90 text-primary-foreground rounded text-sm font-medium transition-colors"
                >
                  Add
                </button>
              </div>
            </div>
          </div>

          <div className="flex gap-2 flex-wrap p-4 bg-muted/5 rounded-lg border border-border">
            {buyInAmounts.map(amount => (
              <button
                key={amount}
                onClick={() => updateSetting('defaultBuyInAmount', amount)}
                className={`text-xs px-3 py-2 rounded border transition-colors group relative ${
                  defaultBuyInAmount === amount
                    ? 'bg-accent border-accent text-primary-foreground'
                    : 'border-border hover:border-accent hover:text-accent'
                }`}
                title="Right-click to remove"
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
            <button
              onClick={() => setShowAllIn(!showAllIn)}
              className={`text-xs px-3 py-2 rounded border transition-colors ${
                showAllIn
                  ? 'bg-accent border-accent text-primary-foreground'
                  : 'border-border hover:border-accent hover:text-accent'
              }`}
            >
              All In
            </button>
          </div>

          <div className="flex gap-2 p-4 bg-muted/5 rounded-lg border border-border overflow-x-auto">
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
                className={`flex items-center gap-1 text-xs px-3 py-2 rounded border whitespace-nowrap transition-colors ${
                  sortBy === metric
                    ? 'bg-accent border-accent text-primary-foreground'
                    : 'border-border hover:border-accent hover:text-accent'
                }`}
              >
                {metric === 'marketCap' && 'MC'}
                {metric === 'athMarketCap' && 'ATH MC'}
                {metric === 'price' && 'Price'}
                {metric === 'holders' && 'Holders'}
                {metric === 'age' && 'Age'}
                {sortBy === metric &&
                  (sortOrder === 'asc' ? (
                    <ChevronUp className="w-3 h-3" />
                  ) : (
                    <ChevronDown className="w-3 h-3" />
                  ))}
              </button>
            ))}
          </div>

          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {sortedCoins.map(coin => (
              <div
                key={coin.symbol}
                className="border border-border rounded-lg p-3 hover:bg-muted/5 transition-colors"
              >
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="font-semibold text-foreground text-sm">{coin.symbol}</div>
                      <div className="text-xs text-muted-foreground">{coin.name}</div>
                      <div className="text-xs text-muted-foreground mt-1">
                        Age: {getCoinAge(coin.createdAt)}
                      </div>
                    </div>
                    <div
                      className={`text-xs font-medium ${coin.change24h >= 0 ? 'text-accent' : 'text-red-500'}`}
                    >
                      {coin.change24h >= 0 ? '+' : ''}
                      {coin.change24h}%
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div className="bg-muted/5 p-2 rounded">
                      <div className="text-muted-foreground">MC</div>
                      <div className="text-foreground font-medium">
                        ${(coin.marketCap / 1000000).toFixed(1)}M
                      </div>
                    </div>
                    <div className="bg-muted/5 p-2 rounded">
                      <div className="text-muted-foreground">ATH MC</div>
                      <div className="text-foreground font-medium">
                        ${(coin.athMarketCap / 1000000).toFixed(1)}M
                      </div>
                    </div>
                    <div className="bg-muted/5 p-2 rounded">
                      <div className="text-muted-foreground">Price</div>
                      <div className="text-foreground font-medium">${coin.price.toFixed(6)}</div>
                    </div>
                    <div className="bg-muted/5 p-2 rounded">
                      <div className="text-muted-foreground">Holders</div>
                      <div className="text-foreground font-medium">
                        {(coin.holders / 1000).toFixed(0)}k
                      </div>
                    </div>
                  </div>

                  <div className="bg-muted/5 p-2 rounded text-xs">
                    <div className="text-muted-foreground">Total Fees</div>
                    <div className="text-foreground font-medium">
                      ${(coin.totalFees / 1000).toFixed(1)}k
                    </div>
                  </div>

                  <div className="flex items-center justify-between gap-2">
                    <div
                      className={`text-xs px-2 py-1 rounded border flex items-center gap-1 ${riskIndicator(coin.riskLevel)}`}
                    >
                      <TrendingUp className="w-3 h-3" />
                      {coin.riskLevel.toUpperCase()}
                    </div>
                    <button
                      onClick={() => handleBuyIn(coin.symbol, defaultBuyInAmount)}
                      className="text-xs px-3 py-1 rounded bg-accent hover:bg-accent/90 text-primary-foreground transition-colors font-medium"
                    >
                      Quick Buy
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
