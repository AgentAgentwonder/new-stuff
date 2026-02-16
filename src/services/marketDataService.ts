import { heliusService, type MemecoinToken } from './heliusService';
import { jupiterService } from './jupiterService';
import { dexScreenerService } from './dexScreenerService';

export interface MarketData {
  solPrice: number;
  tokens: MemecoinToken[];
  topGainers: MemecoinToken[];
  trending: MemecoinToken[];
  lastUpdated: number;
}

export interface CandleData {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

class MarketDataService {
  private solPrice: number = 148.65;
  private tokens: MemecoinToken[] = [];
  private topGainers: MemecoinToken[] = [];
  private trending: MemecoinToken[] = [];
  private lastUpdated: number = 0;
  private subscribers: Set<(data: MarketData) => void> = new Set();

  constructor() {
    // Initialize with mock data
    this.tokens = this.generateInitialTokens();
    this.topGainers = this.tokens.slice(0, 5);
    this.trending = this.tokens.slice(5, 10);
    
    // Start polling for real data
    this.startPolling();
  }

  private generateInitialTokens(): MemecoinToken[] {
    const names = ['DOGE2', 'KEKW', 'LUNAR', 'ASTRO', 'PUMP', 'YOLO', 'CHAD', 'FOMO', 'MOON', 'DIAMOND'];
    return names.map((name, index) => ({
      id: `${name.toLowerCase()}-${Date.now()}-${index}`,
      symbol: name,
      name: `${name} Token`,
      price: Math.random() * 0.001 + 0.00001,
      marketCap: (Math.random() * 0.001 + 0.00001) * (Math.random() * 100000 + 5000),
      liquidity: Math.random() * 50000 + 5000,
      holders: Math.floor(Math.random() * 30) + 5,
      buyTax: Math.floor(Math.random() * 3),
      sellTax: Math.floor(Math.random() * 5),
      riskLevel: 'high' as const,
      age: Math.floor(Math.random() * 86400),
      launchedAt: Date.now() - Math.floor(Math.random() * 86400) * 1000,
      change24h: (Math.random() - 0.5) * 20,
      isNew: Math.random() > 0.7,
      whaleAlert: Math.random() > 0.99,
      volume24h: Math.random() * 100000,
    }));
  }

  private async startPolling(): Promise<void> {
    // Poll for data every 30 seconds
    setInterval(async () => {
      await this.refreshData();
    }, 30000);

    // Initial refresh
    await this.refreshData();
  }

  private async refreshData(): Promise<void> {
    try {
      // Get SOL price
      this.solPrice = await heliusService.getSolPrice();

      // Get new tokens from multiple sources
      const [heliusTokens, dexScreenerPairs] = await Promise.all([
        heliusService.getNewTokens(10),
        dexScreenerService.getRecentPairs(20),
      ]);

      // Combine and deduplicate tokens
      const combinedTokens = this.combineTokenData(heliusTokens, dexScreenerPairs);
      
      // Update market data
      this.tokens = combinedTokens;
      this.topGainers = combinedTokens
        .sort((a, b) => b.change24h - a.change24h)
        .slice(0, 10);
      this.trending = combinedTokens
        .sort((a, b) => b.volume24h - a.volume24h)
        .slice(0, 10);
      
      this.lastUpdated = Date.now();

      // Notify subscribers
      this.notifySubscribers();

    } catch (error) {
      console.error('Failed to refresh market data:', error);
    }
  }

  private combineTokenData(heliusTokens: MemecoinToken[], dexPairs: any[]): MemecoinToken[] {
    const tokenMap = new Map<string, MemecoinToken>();

    // Add Helius tokens
    heliusTokens.forEach(token => {
      tokenMap.set(token.id, token);
    });

    // Add DexScreener data to existing tokens
    dexPairs.forEach(pair => {
      if (pair.tokenAddress) {
        const existing = tokenMap.get(pair.tokenAddress);
        if (existing) {
          tokenMap.set(pair.tokenAddress, {
            ...existing,
            price: pair.price || existing.price,
            marketCap: pair.fdv || existing.marketCap,
            volume24h: pair.volume24h || existing.volume24h,
            change24h: pair.priceChange24h || existing.change24h,
            liquidity: pair.liquidity || existing.liquidity,
          });
        } else {
          // Create new token from DexScreener data
          tokenMap.set(pair.tokenAddress, {
            id: pair.tokenAddress,
            symbol: pair.tokenSymbol || 'UNKNOWN',
            name: pair.tokenName || 'Unknown Token',
            price: pair.price || 0,
            marketCap: pair.fdv || 0,
            liquidity: pair.liquidity || 0,
            holders: Math.floor(Math.random() * 100) + 10,
            buyTax: Math.floor(Math.random() * 5),
            sellTax: Math.floor(Math.random() * 10),
            riskLevel: this.calculateRiskLevel(pair),
            age: Math.floor(Math.random() * 86400),
            launchedAt: Date.now() - Math.floor(Math.random() * 86400) * 1000,
            change24h: pair.priceChange24h || 0,
            isNew: Math.random() > 0.7,
            whaleAlert: Math.random() > 0.99,
            volume24h: pair.volume24h || 0,
            mint: pair.tokenAddress,
          });
        }
      }
    });

    return Array.from(tokenMap.values());
  }

  private calculateRiskLevel(pair: any): 'low' | 'medium' | 'high' {
    const liquidity = pair.liquidity || 0;
    const volume = pair.volume24h || 0;

    if (liquidity < 10000 || volume < 1000) return 'high';
    if (liquidity < 50000 || volume < 5000) return 'medium';
    return 'low';
  }

  // Subscription methods
  subscribe(callback: (data: MarketData) => void): () => void {
    this.subscribers.add(callback);
    
    // Immediately call with current data
    callback(this.getMarketData());
    
    // Return unsubscribe function
    return () => {
      this.subscribers.delete(callback);
    };
  }

  private notifySubscribers(): void {
    const data = this.getMarketData();
    this.subscribers.forEach(callback => {
      callback(data);
    });
  }

  // Public getters
  getMarketData(): MarketData {
    return {
      solPrice: this.solPrice,
      tokens: this.tokens,
      topGainers: this.topGainers,
      trending: this.trending,
      lastUpdated: this.lastUpdated,
    };
  }

  getTokens(): MemecoinToken[] {
    return this.tokens;
  }

  getTokenById(id: string): MemecoinToken | undefined {
    return this.tokens.find(token => token.id === id);
  }

  getTopGainers(): MemecoinToken[] {
    return this.topGainers;
  }

  getTrending(): MemecoinToken[] {
    return this.trending;
  }

  // Generate mock price data for charts
  generateCandleData(basePrice: number, periods: number = 60, intervalMinutes: number = 1): CandleData[] {
    const data: CandleData[] = [];
    const now = Date.now();
    let currentPrice = basePrice;

    for (let i = periods; i >= 0; i--) {
      const time = now - (i * intervalMinutes * 60 * 1000);
      const change = (Math.random() - 0.5) * 0.1 * currentPrice;
      
      const open = currentPrice;
      const close = Math.max(currentPrice + change, 0.00001);
      const high = Math.max(open, close) + Math.random() * 0.05 * currentPrice;
      const low = Math.min(open, close) - Math.random() * 0.05 * currentPrice;
      const volume = Math.random() * 10000;

      data.push({
        time,
        open,
        high,
        low,
        close,
        volume,
      });

      currentPrice = close;
    }

    return data;
  }

  // Add new coin to market data
  addNewCoin(coin: MemecoinToken): void {
    this.tokens = [coin, ...this.tokens].slice(0, 100); // Keep only 100 tokens
    this.notifySubscribers();
  }
}

export const marketDataService = new MarketDataService();