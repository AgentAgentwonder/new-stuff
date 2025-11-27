import { useNavigate, useParams } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { useQuickBuys } from '@/store';
import { Card } from '@/components/ui/card';
import { ArrowLeft } from 'lucide-react';
import CandlestickChart from '@/components/candlestick-chart';

interface CoinData {
  symbol: string;
  name: string;
  price: number;
  marketCap: number;
  holders: number;
  totalFees: number;
  change24h: number;
}

// Mock coin data
const getCoinData = (symbol: string): CoinData => ({
  symbol,
  name: `${symbol} Token`,
  price: 0.000045,
  marketCap: 850000000,
  holders: 125000,
  totalFees: 15000,
  change24h: 12.5,
});

export default function CoinDetailPage() {
  const params = useParams<{ symbol?: string }>();
  const navigate = useNavigate();
  const { amounts: buyInAmounts } = useQuickBuys();
  const coinSymbol = (params.symbol ?? 'SOL').toUpperCase();

  const [coin, setCoin] = useState<CoinData | null>(null);
  const [buyAmount, setBuyAmount] = useState<string>('');
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    setCoin(getCoinData(coinSymbol));
  }, [coinSymbol]);

  if (!mounted || !coin) {
    return <div className="p-8 text-center text-muted-foreground">Loading coin...</div>;
  }

  const handleQuickBuy = (amount: number) => {
    console.log('[v0] Quick buy:', amount, 'of', coinSymbol);
    // Integrate with your trading logic here
  };

  const handleQuickSell = () => {
    console.log('[v0] Quick sell all of', coinSymbol);
    // Integrate with your trading logic here
  };

  const handleCustomBuy = () => {
    const amount = Number.parseFloat(buyAmount);
    if (!isNaN(amount) && amount > 0) {
      console.log('[v0] Custom buy:', amount, 'of', coinSymbol);
      // Integrate with your trading logic here
      setBuyAmount('');
    }
  };

  return (
    <div className="p-4 h-screen overflow-hidden flex flex-col space-y-4 fade-in">
      {/* Header */}
      <div className="flex items-center gap-4">
        <button
          onClick={() => navigate(-1)}
          className="p-2 hover:bg-muted rounded transition-colors"
        >
          <ArrowLeft className="w-5 h-5 text-foreground" />
        </button>
        <div>
          <h1 className="text-3xl font-bold text-foreground">{coin.symbol}</h1>
          <p className="text-muted-foreground">{coin.name}</p>
        </div>
      </div>

      {/* Main Content Grid */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-4 gap-4 overflow-hidden min-h-0">
        {/* Chart and Holders - Takes 3 columns */}
        <div className="lg:col-span-3 flex flex-col gap-4 overflow-hidden">
          {/* Chart */}
          <div className="flex-1 overflow-hidden">
            <CandlestickChart symbol={coin.symbol} />
          </div>

          {/* Holders at Bottom */}
          <Card className="bg-card border-border p-4">
            <div className="grid grid-cols-4 gap-4">
              <div>
                <div className="text-sm text-muted-foreground mb-1">Holders</div>
                <div className="text-2xl font-bold text-foreground">
                  {(coin.holders / 1000).toFixed(0)}k
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground mb-1">Market Cap</div>
                <div className="text-2xl font-bold text-accent">
                  ${(coin.marketCap / 1000000).toFixed(1)}M
                </div>
              </div>
              <div>
                <div className="text-sm text-muted-foreground mb-1">Price</div>
                <div className="text-2xl font-bold text-foreground">${coin.price.toFixed(6)}</div>
              </div>
              <div>
                <div className={`text-sm text-muted-foreground mb-1`}>24h Change</div>
                <div
                  className={`text-2xl font-bold ${coin.change24h >= 0 ? 'text-accent' : 'text-red-500'}`}
                >
                  {coin.change24h >= 0 ? '+' : ''}
                  {coin.change24h}%
                </div>
              </div>
            </div>
          </Card>
        </div>

        {/* Trading Panel - Takes 1 column */}
        <div className="lg:col-span-1 flex flex-col gap-3 overflow-y-auto">
          {/* Price Info */}
          <Card className="bg-card border-border p-3">
            <div className="text-xs text-muted-foreground mb-1">Current Price</div>
            <div className="text-2xl font-bold text-accent">${coin.price.toFixed(6)}</div>
          </Card>

          {/* Quick Sell - Prominent */}
          <button
            onClick={handleQuickSell}
            className="w-full py-3 px-4 bg-red-500 hover:bg-red-600 text-white font-bold rounded transition-colors text-sm"
          >
            QUICK SELL ALL
          </button>

          {/* Quick Buy Amounts */}
          <div>
            <div className="text-xs font-semibold text-muted-foreground mb-2">Quick Buy ($)</div>
            <div className="space-y-2">
              {buyInAmounts.map(amount => (
                <button
                  key={amount}
                  onClick={() => handleQuickBuy(amount)}
                  className="w-full py-2 px-3 bg-accent hover:bg-accent/90 text-primary-foreground font-semibold rounded transition-colors text-sm"
                >
                  Buy ${amount}
                </button>
              ))}
            </div>
          </div>

          {/* Custom Buy Amount */}
          <div className="pt-2 border-t border-border">
            <label className="text-xs font-semibold text-muted-foreground block mb-2">
              Custom Amount ($)
            </label>
            <div className="flex gap-1">
              <input
                type="number"
                value={buyAmount}
                onChange={e => setBuyAmount(e.target.value)}
                placeholder="Enter amount"
                className="flex-1 px-3 py-2 bg-background border border-border rounded text-xs text-foreground"
              />
              <button
                onClick={handleCustomBuy}
                className="px-3 py-2 bg-accent hover:bg-accent/90 text-primary-foreground font-semibold rounded transition-colors text-xs whitespace-nowrap"
              >
                Buy
              </button>
            </div>
          </div>

          {/* All In */}
          <button className="w-full py-2 px-3 border-2 border-accent text-accent font-bold rounded hover:bg-accent/10 transition-colors text-sm">
            ALL IN
          </button>

          {/* Coin Stats */}
          <Card className="bg-muted/5 border-border p-3 text-xs">
            <div className="space-y-1">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Fees</span>
                <span className="text-foreground font-medium">
                  ${(coin.totalFees / 1000).toFixed(1)}k
                </span>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
