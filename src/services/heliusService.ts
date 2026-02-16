import { invoke } from '@tauri-apps/api/core';

export interface HeliusToken {
  account: string;
  mint: string;
  amount: string;
  decimals: number;
  uiAmount: number | null;
  name?: string;
  symbol?: string;
  logoURI?: string;
  verified: boolean;
}

export interface HeliusPrice {
  mint: string;
  price: number;
  change24h: number;
  updatedAt: string;
}

export interface HeliusNewToken {
  mint: string;
  name: string;
  symbol: string;
  description?: string;
  image?: string;
  marketCap: number;
  price: number;
  volume24h: number;
  liquidity: number;
  holders: number;
  age: number;
  createdAt: string;
}

export interface MemecoinToken {
  id: string;
  symbol: string;
  name: string;
  price: number;
  marketCap: number;
  liquidity: number;
  holders: number;
  buyTax: number;
  sellTax: number;
  riskLevel: 'low' | 'medium' | 'high';
  age: number;
  launchedAt: number;
  change24h: number;
  isNew: boolean;
  whaleAlert: boolean;
  volume24h: number;
  mint?: string;
}

class HeliusService {
  private apiKey: string;

  constructor(apiKey: string = 'devnet-helius-api-key-placeholder') {
    this.apiKey = apiKey;
  }

  async getTokenPrices(tokenMints: string[]): Promise<HeliusPrice[]> {
    try {
      const response = await fetch('https://api.helius.xyz/v0/token-metadata', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mintAccounts: tokenMints,
        }),
      });
      
      return await response.json();
    } catch (error) {
      console.error('Failed to fetch token prices:', error);
      return [];
    }
  }

  async getNewTokens(limit: number = 20): Promise<MemecoinToken[]> {
    try {
      // Using Helius API for new token discovery
      const response = await fetch(
        `https://api.helius.xyz/v0/token-metadata?api-key=${this.apiKey}`
      );
      const data = await response.json();
      
      // Transform and filter for memecoins
      return data
        .filter((token: any) => {
          const marketCap = token.market_cap || 0;
          const name = token.name || '';
          return (
            marketCap > 1000 && 
            marketCap < 10000000 && 
            (name.toLowerCase().includes('memecoin') || 
             name.toLowerCase().includes('doge') ||
             name.toLowerCase().includes('shiba') ||
             name.toLowerCase().includes('pepe'))
          );
        })
        .slice(0, limit)
        .map((token: any) => ({
          id: token.mint,
          symbol: token.symbol || 'UNKNOWN',
          name: token.name || 'Unknown Token',
          price: token.price || 0,
          marketCap: token.market_cap || 0,
          liquidity: token.liquidity || 0,
          holders: token.holders || 0,
          buyTax: token.buy_tax || 0,
          sellTax: token.sell_tax || 0,
          riskLevel: this.calculateRiskLevel(token),
          age: token.age || 0,
          launchedAt: token.created_at ? new Date(token.created_at).getTime() : Date.now(),
          change24h: token.price_change_24h || 0,
          isNew: token.age < 86400, // Less than 24 hours
          whaleAlert: token.whale_alerts > 3,
          volume24h: token.volume_24h || 0,
          mint: token.mint,
        }));
    } catch (error) {
      console.error('Failed to fetch new tokens:', error);
      // Return some mock data as fallback
      return this.getMockTokens();
    }
  }

  private calculateRiskLevel(token: any): 'low' | 'medium' | 'high' {
    const liquidity = token.liquidity || 0;
    const holders = token.holders || 0;
    const buyTax = token.buy_tax || 0;
    const sellTax = token.sell_tax || 0;

    if (liquidity < 10000 || holders < 50 || buyTax > 10 || sellTax > 15) {
      return 'high';
    }
    if (liquidity < 50000 || holders < 100 || buyTax > 5 || sellTax > 10) {
      return 'medium';
    }
    return 'low';
  }

  private getMockTokens(): MemecoinToken[] {
    const names = ['DOGE2', 'KEKW', 'LUNAR', 'ASTRO', 'PUMP', 'YOLO', 'CHAD', 'FOMO'];
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

  async subscribeToTokenUpdates(onUpdate: (token: MemecoinToken) => void): Promise<void> {
    // For now, just poll every 5 seconds for new data
    setInterval(async () => {
      try {
        const tokens = await this.getNewTokens(20);
        tokens.forEach(onUpdate);
      } catch (error) {
        console.error('Failed to subscribe to token updates:', error);
      }
    }, 5000);
  }

  // Get wallet token balances
  async getWalletBalances(walletAddress: string): Promise<HeliusToken[]> {
    try {
      const response = await fetch(
        `https://api.helius.xyz/v0/addresses/${walletAddress}/balances?api-key=${this.apiKey}`
      );
      const data = await response.json();
      
      return data.tokens || [];
    } catch (error) {
      console.error('Failed to fetch wallet balances:', error);
      return [];
    }
  }

  // Get SOL price
  async getSolPrice(): Promise<number> {
    try {
      const response = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
      const data = await response.json();
      return data.solana?.usd || 148.65; // Fallback price
    } catch (error) {
      console.error('Failed to fetch SOL price:', error);
      return 148.65; // Fallback price
    }
  }
}

export const heliusService = new HeliusService();