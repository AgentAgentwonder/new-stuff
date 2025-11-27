'use client';

import { useEffect, useState } from 'react';
import {
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  ComposedChart,
  Bar,
} from 'recharts';

interface Candle {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

interface CandlestickChartProps {
  symbol: string;
}

// Generate realistic mock price data
const generateCandleData = (basePrice: number, count: number): Candle[] => {
  const candles: Candle[] = [];
  let currentPrice = basePrice;

  for (let i = 0; i < count; i++) {
    const volatility = (Math.random() - 0.5) * 0.02 * currentPrice;
    const open = currentPrice;
    const close = currentPrice + volatility;
    const high = Math.max(open, close) * (1 + Math.random() * 0.01);
    const low = Math.min(open, close) * (1 - Math.random() * 0.01);
    const volume = Math.floor(Math.random() * 1000000);

    candles.push({
      time: i,
      open,
      high,
      low,
      close,
      volume,
    });

    currentPrice = close;
  }

  return candles;
};

export default function CandlestickChart({ symbol }: CandlestickChartProps) {
  const [candles, setCandles] = useState<Candle[]>([]);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
    setCandles(generateCandleData(0.000045, 100));

    // Update every 50ms for live candlestick updates
    const interval = setInterval(() => {
      setCandles(prev => {
        const updated = [...prev];
        const lastCandle = updated[updated.length - 1];
        const volatility = (Math.random() - 0.5) * 0.02 * lastCandle.close;
        const newClose = lastCandle.close + volatility;

        updated[updated.length - 1] = {
          ...lastCandle,
          close: newClose,
          high: Math.max(lastCandle.high, newClose),
          low: Math.min(lastCandle.low, newClose),
          volume: lastCandle.volume + Math.floor(Math.random() * 10000),
        };

        // Add new candle every 60 updates (3 seconds)
        if (updated.length > 100) {
          updated.shift();
        }

        return updated;
      });
    }, 50);

    return () => clearInterval(interval);
  }, []);

  if (!mounted) {
    return <div className="w-full h-96 bg-muted/5 rounded animate-pulse"></div>;
  }

  const chartData = candles.map(candle => ({
    time: candle.time,
    high: candle.high,
    low: candle.low,
    close: candle.close,
    volume: candle.volume,
  }));

  return (
    <div className="w-full h-96 bg-card border border-border rounded p-4">
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis dataKey="time" stroke="var(--muted-foreground)" />
          <YAxis stroke="var(--muted-foreground)" />
          <Tooltip
            contentStyle={{
              backgroundColor: 'var(--card)',
              border: '1px solid var(--border)',
              borderRadius: '8px',
            }}
            labelStyle={{ color: 'var(--foreground)' }}
          />
          <Line
            type="monotone"
            dataKey="close"
            stroke="var(--accent)"
            strokeWidth={2}
            isAnimationActive={false}
          />
          <Bar dataKey="volume" fill="var(--accent)" opacity={0.2} isAnimationActive={false} />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}
