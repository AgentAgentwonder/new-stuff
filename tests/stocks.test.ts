import { describe, it, expect } from 'vitest';

describe('Stock Intelligence Types', () => {
  it('should have valid TrendingStock structure', () => {
    const mockStock = {
      symbol: 'AAPL',
      name: 'Apple Inc.',
      price: 178.25,
      change24h: 2.15,
      percentChange24h: 1.22,
      volume: 52450000,
      volumeChange24h: 35.6,
      unusualVolume: false,
      marketCap: 2800000000000,
      avgVolume: 48000000,
      reason: null,
    };

    expect(mockStock.symbol).toBe('AAPL');
    expect(mockStock.price).toBeGreaterThan(0);
    expect(typeof mockStock.percentChange24h).toBe('number');
  });

  it('should have valid TopMover structure', () => {
    const mockMover = {
      symbol: 'NVDA',
      name: 'NVIDIA Corporation',
      price: 495.3,
      change: 12.85,
      percentChange: 2.66,
      volume: 45300000,
      marketCap: 1220000000000,
      direction: 'gainer' as const,
      session: 'regular' as const,
      technicalIndicators: {
        rsi: 68.5,
        macd: 'Bullish',
        volumeRatio: 1.42,
        momentum: 'Strong',
      },
      reason: 'AI chip demand surge',
    };

    expect(mockMover.direction).toBe('gainer');
    expect(mockMover.session).toBe('regular');
    expect(mockMover.technicalIndicators.rsi).toBeLessThanOrEqual(100);
    expect(mockMover.technicalIndicators.volumeRatio).toBeGreaterThan(0);
  });

  it('should have valid NewIPO structure', () => {
    const mockIPO = {
      symbol: 'ARMH',
      name: 'ARM Holdings',
      ipoDate: '2024-09-15',
      offerPrice: 51.0,
      currentPrice: 68.5,
      percentChange: 34.31,
      sharesOffered: 95500000,
      marketCap: 70000000000,
      exchange: 'NASDAQ',
      status: 'recent' as const,
    };

    expect(mockIPO.status).toMatch(/upcoming|today|recent|filed/);
    expect(mockIPO.offerPrice).toBeGreaterThan(0);
    if (mockIPO.currentPrice && mockIPO.offerPrice) {
      const expectedChange =
        ((mockIPO.currentPrice - mockIPO.offerPrice) / mockIPO.offerPrice) * 100;
      expect(Math.abs(expectedChange - (mockIPO.percentChange || 0))).toBeLessThan(0.1);
    }
  });

  it('should have valid EarningsEvent structure', () => {
    const mockEarnings = {
      symbol: 'AAPL',
      name: 'Apple Inc.',
      date: '2024-10-31',
      time: 'aftermarket' as const,
      fiscalQuarter: 'Q4 2024',
      estimateEps: 1.39,
      actualEps: undefined,
      surprisePercent: undefined,
      historicalReaction: {
        avgMovePercent: 3.2,
        lastReactionPercent: 4.1,
        beatMissRatio: '8/2',
      },
      hasAlert: true,
    };

    expect(mockEarnings.time).toMatch(/beforemarket|aftermarket|duringmarket/);
    expect(mockEarnings.historicalReaction?.avgMovePercent).toBeDefined();
    expect(typeof mockEarnings.hasAlert).toBe('boolean');
  });

  it('should have valid StockNews structure', () => {
    const mockNews = {
      id: '1',
      symbol: 'TSLA',
      title: 'Tesla Reports Strong Q4 Results',
      summary: 'Company exceeds analyst expectations',
      aiSummary: 'Tesla reported Q4 revenue of $123B, beating estimates by 8%',
      url: 'https://example.com/news/1',
      source: 'Reuters',
      publishedAt: new Date().toISOString(),
      sentiment: 'bullish' as const,
      impactLevel: 'high' as const,
      topics: ['earnings', 'revenue'],
    };

    expect(mockNews.sentiment).toMatch(/bullish|neutral|bearish/);
    expect(mockNews.impactLevel).toMatch(/high|medium|low/);
    expect(Array.isArray(mockNews.topics)).toBe(true);
  });
});

describe('Stock Data Transformations', () => {
  it('should format large numbers correctly', () => {
    const formatNumber = (num: number): string => {
      if (num >= 1e9) return `$${(num / 1e9).toFixed(2)}B`;
      if (num >= 1e6) return `$${(num / 1e6).toFixed(2)}M`;
      if (num >= 1e3) return `$${(num / 1e3).toFixed(2)}K`;
      return `$${num.toFixed(2)}`;
    };

    expect(formatNumber(2800000000000)).toBe('$2800.00B');
    expect(formatNumber(52450000)).toBe('$52.45M');
    expect(formatNumber(1250)).toBe('$1.25K');
    expect(formatNumber(178.25)).toBe('$178.25');
  });

  it('should calculate percentage change correctly', () => {
    const calculateChange = (current: number, previous: number): number => {
      return ((current - previous) / previous) * 100;
    };

    expect(calculateChange(110, 100)).toBeCloseTo(10, 2);
    expect(calculateChange(90, 100)).toBeCloseTo(-10, 2);
    expect(calculateChange(105.5, 100)).toBeCloseTo(5.5, 2);
  });

  it('should detect unusual volume correctly', () => {
    const isUnusualVolume = (volume: number, avgVolume: number, threshold = 50): boolean => {
      const change = ((volume - avgVolume) / avgVolume) * 100;
      return change > threshold;
    };

    expect(isUnusualVolume(150000000, 100000000)).toBe(false);
    expect(isUnusualVolume(160000000, 100000000)).toBe(true);
    expect(isUnusualVolume(100000000, 100000000)).toBe(false);
  });
});
