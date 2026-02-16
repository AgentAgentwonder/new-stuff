export interface DexScreenerPair {
  pairAddress: string;
  dexId: string;
  tokenSymbol: string;
  tokenName: string;
  tokenAddress: string;
  quoteSymbol: string;
  price: number;
  priceChange24h: number;
  liquidity: number;
  fdv: number;
  volume24h: number;
  transactions24h: {
    buys: number;
    sells: number;
  };
  createdAt: number;
}

export interface DexScreenerTokenData {
  address: string;
  symbol: string;
  name: string;
  price: string;
  priceChange24h: {
    h1: number;
    h6: number;
    h24h: number;
  };
  liquidity: {
    usd: number;
  };
  fdv: number;
  pairAddress: string;
  markets: DexScreenerPair[];
}

class DexScreenerService {
  private baseUrl = 'https://api.dexscreener.com';

  async getTokenData(tokenAddress: string): Promise<DexScreenerTokenData | null> {
    try {
      const response = await fetch(`${this.baseUrl}/tokens/v1/${tokenAddress}`);
      
      if (!response.ok) {
        console.error('Token data request failed:', response.statusText);
        return null;
      }
      
      const data = await response.json();
      return data?.[0] || null;
    } catch (error) {
      console.error('Failed to get token data:', error);
      return null;
    }
  }

  async getPairs(tokenAddresses: string[]): Promise<DexScreenerTokenData[]> {
    try {
      const addresses = tokenAddresses.join(',');
      const response = await fetch(`${this.baseUrl}/tokens/v1/${addresses}`);
      
      if (!response.ok) {
        console.error('Pairs request failed:', response.statusText);
        return [];
      }
      
      return await response.json();
    } catch (error) {
      console.error('Failed to get pairs:', error);
      return [];
    }
  }

  async getRecentPairs(limit: number = 20): Promise<DexScreenerPair[]> {
    try {
      const response = await fetch(`${this.baseUrl}/pairs/latest?limit=${limit}`);
      
      if (!response.ok) {
        console.error('Recent pairs request failed:', response.statusText);
        return [];
      }
      
      const data = await response.json();
      return data.pairs || [];
    } catch (error) {
      console.error('Failed to get recent pairs:', error);
      return [];
    }
  }

  // Get top gainers
  async getTopGainers(): Promise<DexScreenerPair[]> {
    try {
      const response = await fetch(`${this.baseUrl}/pairs/latest?sort=priceChange24h&order=desc&limit=20`);
      
      if (!response.ok) {
        console.error('Top gainers request failed:', response.statusText);
        return [];
      }
      
      const data = await response.json();
      return data.pairs || [];
    } catch (error) {
      console.error('Failed to get top gainers:', error);
      return [];
    }
  }

  // Get trending tokens
  async getTrending(): Promise<DexScreenerPair[]> {
    try {
      const response = await fetch(`${this.baseUrl}/tokens/trending`);
      
      if (!response.ok) {
        console.error('Trending request failed:', response.statusText);
        return [];
      }
      
      const data = await response.json();
      return data.tokens || [];
    } catch (error) {
      console.error('Failed to get trending:', error);
      return [];
    }
  }

  // Calculate token metrics
  calculateRiskScore(pair: DexScreenerPair): 'low' | 'medium' | 'high' {
    const liquidity = pair.liquidity || 0;
    const volume = pair.volume24h || 0;
    const buys = pair.transactions24h?.buys || 0;
    const sells = pair.transactions24h?.sells || 0;
    const buySellRatio = buys / (sells + 1);

    // High risk indicators
    if (liquidity < 10000) return 'high';
    if (volume < 1000) return 'high';
    if (buySellRatio < 0.3) return 'high';
    if (buySellRatio > 10) return 'high'; // Suspiciously high buys

    // Medium risk indicators
    if (liquidity < 50000) return 'medium';
    if (volume < 5000) return 'medium';
    if (buySellRatio < 0.5 || buySellRatio > 5) return 'medium';

    return 'low';
  }
}

export const dexScreenerService = new DexScreenerService();