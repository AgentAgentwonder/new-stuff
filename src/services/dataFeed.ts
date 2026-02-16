import { Memecoin, PriceUpdate } from '../types/trading';
import { useTradingStore } from '../store/tradingStore';

// Free Helius API endpoint (rate limited)
const HELIUS_RPC = 'https://mainnet.helius-rpc.com/?api-key=';

// Jupiter API for price data
const JUPITER_PRICE_API = 'https://price.jup.ag/v6/price';

// WebSocket connections for real-time data
export class DataFeedService {
  private wsConnections: Map<string, WebSocket> = new Map();
  private pollingIntervals: Map<string, number> = new Map();
  private heliusKey: string | null = null;

  setHeliusKey(key: string): void {
    this.heliusKey = key;
  }

  async fetchTokenData(mintAddress: string): Promise<Memecoin | null> {
    try {
      // Try Jupiter first (free, no key needed)
      const jupiterData = await this.fetchJupiterData(mintAddress);
      if (jupiterData) return jupiterData;

      // Fallback to Helius if key provided
      if (this.heliusKey) {
        const heliusData = await this.fetchHeliusData(mintAddress);
        if (heliusData) return heliusData;
      }

      return null;
    } catch (error) {
      console.error('Failed to fetch token data:', error);
      return null;
    }
  }

  private async fetchJupiterData(mintAddress: string): Promise<Memecoin | null> {
    try {
      const response = await fetch(`${JUPITER_PRICE_API}?ids=${mintAddress}`);
      if (!response.ok) return null;

      const data = await response.json();
      const priceData = data.data?.[mintAddress];

      if (!priceData) return null;

      // Fetch additional metadata from Solana FM or other free sources
      const metadata = await this.fetchTokenMetadata(mintAddress);

      return {
        address: mintAddress,
        symbol: metadata?.symbol || 'UNKNOWN',
        name: metadata?.name || 'Unknown Token',
        decimals: metadata?.decimals || 9,
        priceUsd: priceData.price,
        priceChange24h: priceData.priceChange24h || 0,
        volume24h: 0, // Jupiter doesn't provide volume in free tier
        marketCap: 0,
        liquidity: 0,
        holderCount: 0,
        createdAt: 0,
        isVerified: false,
        isMutable: true,
        mintAuthority: null,
        freezeAuthority: null,
        supply: 0,
        circulatingSupply: 0,
        lpBurned: false,
        topHolders: [],
        recentTransactions: [],
        lastUpdated: Date.now(),
      };
    } catch {
      return null;
    }
  }

  private async fetchHeliusData(mintAddress: string): Promise<Memecoin | null> {
    if (!this.heliusKey) return null;

    try {
      const response = await fetch(`${HELIUS_RPC}${this.heliusKey}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'getAsset',
          params: { id: mintAddress },
        }),
      });

      if (!response.ok) return null;
      const data = await response.json();

      // Parse Helius response
      const asset = data.result;
      if (!asset) return null;

      return {
        address: mintAddress,
        symbol: asset.token_info?.symbol || 'UNKNOWN',
        name: asset.content?.metadata?.name || 'Unknown Token',
        decimals: asset.token_info?.decimals || 9,
        priceUsd: asset.token_info?.price_info?.price_per_token || 0,
        priceChange24h: 0,
        volume24h: 0,
        marketCap: 0,
        liquidity: 0,
        holderCount: asset.ownership?.owner || 0,
        createdAt: asset.minting?.timestamp || 0,
        isVerified: false,
        isMutable: asset.mutable || true,
        mintAuthority: asset.authorities?.[0]?.address || null,
        freezeAuthority: null,
        supply: asset.token_info?.supply || 0,
        circulatingSupply: asset.token_info?.supply || 0,
        lpBurned: false,
        topHolders: [],
        recentTransactions: [],
        lastUpdated: Date.now(),
      };
    } catch {
      return null;
    }
  }

  private async fetchTokenMetadata(mintAddress: string): Promise<{ symbol: string; name: string; decimals: number } | null> {
    // Try to fetch from Solana token list or Metaplex
    try {
      const response = await fetch(`https://cdn.jsdelivr.net/gh/solana-labs/token-list@main/src/tokens/solana.tokenlist.json`);
      const data = await response.json();
      const token = data.tokens.find((t: { address: string }) => t.address === mintAddress);
      if (token) {
        return {
          symbol: token.symbol,
          name: token.name,
          decimals: token.decimals,
        };
      }
    } catch {
      // Fallback to generic
    }
    return null;
  }

  startPriceFeed(addresses: string[], onPriceUpdate: (update: PriceUpdate) => void): void {
    // Use polling instead of WebSocket for free tier
    addresses.forEach((address) => {
      // Clear existing interval
      if (this.pollingIntervals.has(address)) {
        window.clearInterval(this.pollingIntervals.get(address));
      }

      // Poll every 3 seconds for near real-time updates
      const interval = window.setInterval(async () => {
        try {
          const response = await fetch(`${JUPITER_PRICE_API}?ids=${address}`);
          if (!response.ok) return;

          const data = await response.json();
          const priceData = data.data?.[address];

          if (priceData) {
            onPriceUpdate({
              address,
              priceUsd: priceData.price,
              timestamp: Date.now(),
              source: 'jupiter',
            });

            // Update store
            useTradingStore.getState().updatePrice(address, priceData.price);
          }
        } catch (error) {
          console.error('Price poll failed:', error);
        }
      }, 3000);

      this.pollingIntervals.set(address, interval);
    });
  }

  stopPriceFeed(address?: string): void {
    if (address) {
      const interval = this.pollingIntervals.get(address);
      if (interval) {
        window.clearInterval(interval);
        this.pollingIntervals.delete(address);
      }
    } else {
      // Stop all
      this.pollingIntervals.forEach((interval) => window.clearInterval(interval));
      this.pollingIntervals.clear();
    }
  }

  watchNewTokens(_onNewToken: (coin: Memecoin) => void): void {
    // Monitor pump.fun or similar launchpads for new tokens
    // This requires paid APIs or custom scraping, skipping for free tier
    console.log('New token monitoring requires paid API or custom implementation');
  }

  destroy(): void {
    this.stopPriceFeed();
    this.wsConnections.forEach((ws) => ws.close());
    this.wsConnections.clear();
  }
}

export const dataFeed = new DataFeedService();
