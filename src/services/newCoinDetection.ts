import { Memecoin, TradeSignal } from '../types/trading';
import { AITradingEngine } from './aiTrading';
import { HeliusWebSocketService } from './heliusWebSocket';

// Token list disabled - GitHub endpoint 404, Jupiter API CORS-blocked
// Using DexScreener + Helius WebSocket instead for detection

// DexScreener for new pairs
const DEXSCREENER_API = 'https://api.dexscreener.com/latest';

// These are unused - Pump.fun has no public API
// const PUMPFUN_API = 'https://frontend-api.pump.fun';
// const PUMPFUN_COINS = `${PUMPFUN_API}/coins/for-you`;

// Helius DAS API for real-time monitoring
const HELIUS_RPC = 'https://mainnet.helius-rpc.com/?api-key=';

// Unused - all RPC calls should use Helius API key for reliability
// const QUICKNODE_RPC = 'https://api.mainnet-beta.solana.com';

export interface WhaleActivity {
  totalBuyVolume: number;
  totalSellVolume: number;
  buyPressureRatio: number;
  largeBuys: { amount: number; time: number }[]; // Buys >$1000
  whales: number; // Count of unique wallets with >$1000 buys
  whaleAlert: 'none' | 'low' | 'medium' | 'high' | 'extreme';
  totalInvestedUsd: number;
  lastUpdated: number;
}

export interface NewCoinDetection {
  coin: Memecoin;
  detectedAt: number;
  signal: TradeSignal;
  source: 'pumpfun' | 'dexscreener' | 'helius' | 'jupiter';
  ageSeconds: number;
  whaleActivity: WhaleActivity;
}

export class NewCoinDetectionService {
  private aiEngine: AITradingEngine;
  private heliusService: HeliusWebSocketService | null = null;
  private newCoins: Map<string, NewCoinDetection> = new Map();
  private onNewCoinCallbacks: ((coin: NewCoinDetection) => void)[] = [];
  private onWhaleAlertCallbacks: ((coin: NewCoinDetection) => void)[] = [];
  private pollingIntervals: number[] = [];
  private isRunning: boolean = false;
  private lastProcessedTokens: Set<string> = new Set();
  private readonly MAX_HISTORY = 1000;
  private readonly DISPLAY_LIMIT = 10;
  private readonly WHALE_THRESHOLD = 1000; // $1000 minimum for whale detection
  private heliusApiKey: string | null = null;

  constructor(aiEngine: AITradingEngine, heliusApiKey?: string) {
    this.aiEngine = aiEngine;
    if (heliusApiKey) {
      this.setHeliusApiKey(heliusApiKey);
    }
  }

  setHeliusApiKey(apiKey: string): void {
    this.heliusApiKey = apiKey;
    // If already running, restart with Helius
    if (this.isRunning) {
      this.stop();
      this.start();
    }
  }

  start(): void {
    if (this.isRunning) return;
    this.isRunning = true;

    console.log('Starting new coin detection service...');

    // Use Helius WebSocket if API key available (fastest - 1-2s)
    if (this.heliusApiKey) {
      console.log('Using Helius WebSocket for real-time detection (1-2s latency)');
      this.heliusService = new HeliusWebSocketService({ apiKey: this.heliusApiKey });
      
      // Subscribe to new token mints
      this.heliusService.onTokenMint((event) => {
        this.handleHeliusTokenMint(event);
      });

      // Subscribe to large trades for whale detection
      this.heliusService.onLargeTrade((data) => {
        this.handleHeliusLargeTrade(data);
      });

      this.heliusService.start();
    } else {
      console.log('No Helius API key - falling back to DexScreener (slower)');
    }

    // Fallback polling - much more aggressive for sub-5s detection
    // DexScreener - new pairs (aggressive 2s polling)
    this.pollingIntervals.push(
      window.setInterval(() => this.pollDexScreener(), 2000) // 2 seconds
    );

    // Jupiter token list updates - DISABLED (endpoint broken)
    // this.pollingIntervals.push(
    //   window.setInterval(() => this.pollJupiter(), 3000)
    // );

    // Rapid initial polling burst (first 30 seconds)
    let rapidPollCount = 0;
    const rapidInterval = window.setInterval(() => {
      rapidPollCount++;
      this.pollDexScreener();
      if (rapidPollCount >= 15) {
        window.clearInterval(rapidInterval);
      }
    }, 1000); // Every 1s for first 15s
    this.pollingIntervals.push(rapidInterval);

    // Initial poll
    this.pollAllSources();
  }

  stop(): void {
    this.isRunning = false;
    this.pollingIntervals.forEach(id => window.clearInterval(id));
    this.pollingIntervals = [];
    
    if (this.heliusService) {
      this.heliusService.stop();
      this.heliusService = null;
    }
  }

  onNewCoin(callback: (coin: NewCoinDetection) => void): () => void {
    this.onNewCoinCallbacks.push(callback);
    return () => {
      const index = this.onNewCoinCallbacks.indexOf(callback);
      if (index > -1) this.onNewCoinCallbacks.splice(index, 1);
    };
  }

  onWhaleAlert(callback: (coin: NewCoinDetection) => void): () => void {
    this.onWhaleAlertCallbacks.push(callback);
    return () => {
      const index = this.onWhaleAlertCallbacks.indexOf(callback);
      if (index > -1) this.onWhaleAlertCallbacks.splice(index, 1);
    };
  }

  // Check for whale activity on a coin
  async checkWhaleActivity(address: string): Promise<WhaleActivity | null> {
    const detection = this.newCoins.get(address);
    if (!detection) return null;

    try {
      // Fetch recent transactions for this coin
      const transactions = await this.fetchRecentTransactions(address);
      
      const whaleActivity = this.analyzeWhaleActivity(transactions);
      
      // Update detection
      detection.whaleActivity = whaleActivity;
      this.newCoins.set(address, detection);

      // Trigger alert if significant whale activity
      if (whaleActivity.whaleAlert !== 'none') {
        this.onWhaleAlertCallbacks.forEach(cb => {
          try {
            cb(detection);
          } catch (e) {
            console.error('Whale alert callback error:', e);
          }
        });
      }

      return whaleActivity;
    } catch (error) {
      console.error('Failed to check whale activity:', error);
      return null;
    }
  }

  private async fetchRecentTransactions(_address: string): Promise<any[]> {
    // This would integrate with Helius or other APIs to get real transactions
    // For now, return empty array - real implementation would parse blockchain data
    return [];
  }

  private analyzeWhaleActivity(transactions: any[]): WhaleActivity {
    const largeBuys: { amount: number; time: number }[] = [];
    const uniqueWallets = new Set<string>();
    let totalBuyVolume = 0;
    let totalSellVolume = 0;
    let totalInvested = 0;

    for (const tx of transactions) {
      const value = tx.valueUsd || 0;
      const isBuy = tx.type === 'buy';
      
      if (isBuy) {
        totalBuyVolume += value;
        if (value >= this.WHALE_THRESHOLD) {
          largeBuys.push({ amount: value, time: tx.timestamp || Date.now() });
          uniqueWallets.add(tx.wallet || tx.sender);
          totalInvested += value;
        }
      } else {
        totalSellVolume += value;
      }
    }

    // Sort large buys by amount (descending)
    largeBuys.sort((a, b) => b.amount - a.amount);

    // Calculate buy pressure ratio
    const buyPressureRatio = totalSellVolume > 0 
      ? totalBuyVolume / totalSellVolume 
      : totalBuyVolume > 0 ? Infinity : 1;

    // Determine whale alert level
    let whaleAlert: WhaleActivity['whaleAlert'] = 'none';
    const totalWhaleVolume = largeBuys.reduce((sum, b) => sum + b.amount, 0);
    
    if (totalWhaleVolume >= 50000) {
      whaleAlert = 'extreme';
    } else if (totalWhaleVolume >= 20000) {
      whaleAlert = 'high';
    } else if (totalWhaleVolume >= 5000) {
      whaleAlert = 'medium';
    } else if (totalWhaleVolume >= 1000) {
      whaleAlert = 'low';
    }

    return {
      totalBuyVolume,
      totalSellVolume,
      buyPressureRatio,
      largeBuys: largeBuys.slice(0, 10), // Keep top 10
      whales: uniqueWallets.size,
      whaleAlert,
      totalInvestedUsd: totalInvested,
      lastUpdated: Date.now(),
    };
  }

  getTop10Newest(): NewCoinDetection[] {
    return Array.from(this.newCoins.values())
      .sort((a, b) => b.detectedAt - a.detectedAt)
      .slice(0, this.DISPLAY_LIMIT);
  }

  private async pollAllSources(): Promise<void> {
    await Promise.all([
      this.pollPumpFun(),
      this.pollDexScreener(),
      // this.pollJupiter(), // DISABLED - endpoint broken
      this.pollHelius(),
    ]).catch(console.error);
  }

  // Poll Pump.fun for latest launches (use DexScreener instead since Pump.fun blocks CORS)
  private async pollPumpFun(): Promise<void> {
    // Pump.fun doesn't have a public CORS-enabled API
    // We'll rely on DexScreener for new pairs instead
    // This method is kept for compatibility but disabled
    return;
  }

  // Poll DexScreener for new Solana pairs (our primary source)
  private async pollDexScreener(): Promise<void> {
    try {
      // Get newest pairs on Solana - look for recently created pairs
      const response = await fetch(
        `${DEXSCREENER_API}/dex/search?q=solana`,
        {
          headers: {
            'Accept': 'application/json',
          },
        }
      );

      if (!response.ok) {
        console.debug('DexScreener fetch failed:', response.status);
        return;
      }

      const data = await response.json();
      
      if (!data.pairs || !Array.isArray(data.pairs)) {
        console.debug('DexScreener response missing pairs');
        return;
      }

      // Filter for very new pairs (less than 2 hours old) or high activity
      const twoHoursAgo = Date.now() - (2 * 60 * 60 * 1000);
      
      for (const pair of data.pairs.slice(0, 50)) {
        // Skip if not Solana
        if (pair.chainId !== 'solana') continue;

        const baseToken = pair.baseToken;
        if (!baseToken || !baseToken.address) continue;

        // Skip if already processed
        if (this.lastProcessedTokens.has(baseToken.address)) continue;

        // Check if pair is new OR has high volume (indicating fresh activity)
        const pairCreated = pair.pairCreatedAt;
        const isNew = !pairCreated || pairCreated > twoHoursAgo;
        const hasHighVolume = (pair.volume?.h24 || 0) > 5000; // $5K+ volume
        
        if (!isNew && !hasHighVolume) continue; // Skip old low-activity pairs

        // Skip wrapped SOL and known stablecoins
        if (this.isStablecoinOrWrapper(baseToken.address, baseToken.symbol, baseToken.name)) continue;
        
        // Skip if quote token is also SOL (likely a SOL pair)
        if (pair.quoteToken?.address && this.isStablecoinOrWrapper(pair.quoteToken.address)) continue;

        const memecoin = this.convertDexScreenerToMemecoin(pair, baseToken);
        if (memecoin) {
          this.processNewCoin(memecoin, 'dexscreener');
        }
      }
    } catch (error) {
      console.debug('DexScreener poll failed:', error);
    }
  }

  // Poll Jupiter token list - DISABLED (endpoint broken)
  // private async pollJupiter(): Promise<void> {
  //   try {
  //     const response = await fetch(JUPITER_TOKEN_LIST, {
  //       headers: {
  //         'Accept': 'application/json',
  //       },
  //     });

  //     if (!response.ok) {
  //       console.debug('Jupiter token list fetch failed:', response.status);
  //       return;
  //     }

  //     const data = await response.json();
      
  //     if (!data.tokens || !Array.isArray(data.tokens)) {
  //       // Try alternative format
  //       if (!Array.isArray(data)) {
  //         console.debug('Jupiter token list format unexpected');
  //         return;
  //       }
  //     }

  //     const tokens = data.tokens || data;
      
  //     // Look for tokens with recent activity or new additions
  //     // Since we can't reliably get 'createdAt', we'll check for high volume activity
  //     for (const token of tokens.slice(0, 100)) {
  //       if (!token.address || this.lastProcessedTokens.has(token.address)) continue;

  //       // Skip SOL and wrapped variants by symbol/name
  //       if (this.isStablecoinOrWrapper(token.address, token.symbol, token.name)) continue;

  //       // Skip if no market data available
  //       if (!token.daily_volume && !token.market_cap) continue;

  //       // Only process if there's significant recent activity
  //       const hasActivity = (token.daily_volume || 0) > 1000 || (token.market_cap || 0) > 10000;
  //       if (!hasActivity) continue;

  //       const memecoin = this.convertJupiterToMemecoin(token);
  //       if (memecoin) {
  //         this.processNewCoin(memecoin, 'jupiter');
  //       }
  //     }
  //   } catch (error) {
  //     console.debug('Jupiter poll failed:', error);
  //   }
  // }

  // Poll Helius for new mint accounts
  private async pollHelius(): Promise<void> {
    // This requires a paid Helius API key for real-time monitoring
    // Free tier can still poll but less frequently
    const apiKey = ''; // User needs to provide their own
    
    if (!apiKey) {
      // Skip if no API key
      return;
    }

    try {
      const response = await fetch(`${HELIUS_RPC}${apiKey}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'getSignaturesForAddress',
          params: [
            'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', // Token program
            { limit: 20 },
          ],
        }),
      });

      if (!response.ok) return;

      const data = await response.json();
      
      if (!data.result || !Array.isArray(data.result)) return;

      for (const sig of data.result) {
        // Get transaction details to find new mints
        await this.processHeliusSignature(sig.signature);
      }
    } catch (error) {
      console.debug('Helius poll failed:', error);
    }
  }

  private async processHeliusSignature(_signature: string): Promise<void> {
    // Would need transaction parsing to extract new mints
    // This is a placeholder for the implementation
  }

  private processNewCoin(coin: Memecoin, source: 'pumpfun' | 'dexscreener' | 'helius' | 'jupiter'): void {
    // Add to processed set
    this.lastProcessedTokens.add(coin.address);

    // Limit history size
    if (this.lastProcessedTokens.size > this.MAX_HISTORY) {
      const iterator = this.lastProcessedTokens.values();
      const first = iterator.next().value;
      if (first) this.lastProcessedTokens.delete(first);
    }

    // Calculate risk signal immediately
    const signal = this.aiEngine.analyzeCoinWithLearning(coin);

    const initialWhaleActivity: WhaleActivity = {
      totalBuyVolume: 0,
      totalSellVolume: 0,
      buyPressureRatio: 1,
      largeBuys: [],
      whales: 0,
      whaleAlert: 'none',
      totalInvestedUsd: 0,
      lastUpdated: Date.now(),
    };

    const detection: NewCoinDetection = {
      coin,
      detectedAt: Date.now(),
      signal,
      source,
      ageSeconds: 0,
      whaleActivity: initialWhaleActivity,
    };

    // Store
    this.newCoins.set(coin.address, detection);

    // Cleanup old coins (keep last 50)
    if (this.newCoins.size > 50) {
      const sorted = Array.from(this.newCoins.entries())
        .sort((a, b) => b[1].detectedAt - a[1].detectedAt);
      const toDelete = sorted.slice(50);
      toDelete.forEach(([key]) => this.newCoins.delete(key));
    }

    // Notify callbacks
    this.onNewCoinCallbacks.forEach(cb => {
      try {
        cb(detection);
      } catch (e) {
        console.error('New coin callback error:', e);
      }
    });

    console.log(`New coin detected [${source}]: ${coin.symbol} - Signal: ${signal.signal}`);
  }

  private convertPumpFunToMemecoin(coin: any): Memecoin | null {
    try {
      return {
        address: coin.mint || coin.address,
        symbol: coin.symbol || 'UNKNOWN',
        name: coin.name || coin.symbol || 'Unknown Token',
        decimals: coin.decimals || 9,
        priceUsd: coin.usd_market_cap ? coin.usd_market_cap / (coin.total_supply || 1_000_000_000) : 0,
        priceChange24h: coin.price_change_24h || 0,
        volume24h: coin.volume_24h || 0,
        marketCap: coin.usd_market_cap || 0,
        liquidity: coin.liquidity || 0,
        holderCount: coin.holder_count || 0,
        createdAt: coin.created_timestamp || Date.now(),
        isVerified: false,
        isMutable: coin.mutable !== false,
        mintAuthority: coin.mint_authority || null,
        freezeAuthority: coin.freeze_authority || null,
        supply: coin.total_supply || 0,
        circulatingSupply: coin.total_supply || 0,
        lpBurned: coin.lp_burned || false,
        topHolders: [],
        recentTransactions: [],
        lastUpdated: Date.now(),
      };
    } catch (e) {
      console.error('Failed to convert pump.fun coin:', e);
      return null;
    }
  }

  private convertDexScreenerToMemecoin(pair: any, token: any): Memecoin | null {
    try {
      const priceUsd = parseFloat(pair.priceUsd || pair.priceNative || '0');
      const marketCap = parseFloat(pair.marketCap || pair.fdv || '0');
      const volume24h = parseFloat(pair.volume?.h24 || '0');
      const liquidity = parseFloat(pair.liquidity?.usd || '0');
      const priceChange24h = parseFloat(pair.priceChange?.h24 || '0');

      return {
        address: token.address,
        symbol: token.symbol || 'UNKNOWN',
        name: token.name || token.symbol || 'Unknown Token',
        decimals: token.decimals || 9,
        priceUsd,
        priceChange24h,
        volume24h,
        marketCap,
        liquidity,
        holderCount: 0, // Not available from DexScreener
        createdAt: pair.pairCreatedAt || Date.now(),
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
    } catch (e) {
      console.error('Failed to convert DexScreener coin:', e);
      return null;
    }
  }

  private convertJupiterToMemecoin(token: any): Memecoin | null {
    try {
      return {
        address: token.address,
        symbol: token.symbol || 'UNKNOWN',
        name: token.name || token.symbol || 'Unknown Token',
        decimals: token.decimals || 9,
        priceUsd: 0, // Jupiter list doesn't include price
        priceChange24h: 0,
        volume24h: 0,
        marketCap: 0,
        liquidity: 0,
        holderCount: 0,
        createdAt: token.createdAt || Date.now(),
        isVerified: token.tags?.includes('verified') || false,
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
    } catch (e) {
      console.error('Failed to convert Jupiter token:', e);
      return null;
    }
  }

  private isStablecoinOrWrapper(address: string, symbol?: string, name?: string): boolean {
    const knownAddresses = [
      'So11111111111111111111111111111111111111112', // Wrapped SOL
      'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
      'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB', // USDT
      'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263', // BONK
      '7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARo', // stSOL
      'mSoLzYCxHdYgdzU16g5QSh3i5K3z3i8kBzpjL8Z2y8',  // mSOL
      'bSo13r4TkiE4xumyjwWj7KBEi6wxm5Ty8fxwNhXz7',    // bSOL
      'J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCP',  // jitoSOL
      '27G8MtK7VtuxCH5UASZ1HXh7b9EJ9W1Q2K9Kz3P5',     // Other SOL variants
    ];
    
    // Check address
    if (knownAddresses.includes(address)) return true;
    
    // Check symbol/name for SOL variants
    const solVariants = ['SOL', 'WSOL', 'SOLANA', 'wSOL', 'mSOL', 'stSOL', 'bSOL', 'jitoSOL'];
    const checkSymbol = (symbol || '').toUpperCase();
    const checkName = (name || '').toUpperCase();
    
    // Reject if it's clearly SOL or a wrapped variant
    if (solVariants.includes(checkSymbol)) return true;
    if (solVariants.some(s => checkName.includes(s))) return true;
    if (checkName.includes('WRAPPED SOL') || checkName.includes('SOLANA')) return true;
    
    return false;
  }

  // Handle Helius WebSocket token mint events (real-time)
  private async handleHeliusTokenMint(event: { signature: string; mint: string; timestamp: number }): Promise<void> {
    if (!event.mint || this.lastProcessedTokens.has(event.mint)) return;
    
    // Skip SOL mints (not real tokens, just SOL transfers)
    if (event.mint === 'So11111111111111111111111111111111111111112') {
      console.log('Skipping SOL mint event (not a new token)');
      return;
    }

    console.log(`Helius detected new token mint: ${event.mint}`);

    try {
      // Fetch detailed metadata from Helius
      const metadata = this.heliusService 
        ? await this.heliusService.getTokenMetadata(event.mint)
        : null;
        
      // Skip if metadata reveals it's SOL or a variant
      if (metadata && this.isStablecoinOrWrapper(event.mint, metadata.symbol, metadata.name)) {
        console.log(`⛔ FILTERED: ${metadata.symbol || event.mint} - SOL/Stablecoin variant`);
        return;
      }

      console.log(`✅ PROCESSING: ${metadata?.symbol || 'NEW TOKEN'} (${event.mint.slice(0, 8)}...)`);

      // Create memecoin object
      const memecoin: Memecoin = {
        address: event.mint,
        symbol: metadata?.symbol || 'NEW',
        name: metadata?.name || 'New Token',
        decimals: metadata?.decimals || 9,
        priceUsd: 0, // Will update from DEX
        priceChange24h: 0,
        volume24h: 0,
        marketCap: 0,
        liquidity: 0,
        holderCount: 0,
        createdAt: event.timestamp,
        isVerified: false,
        isMutable: true,
        mintAuthority: null,
        freezeAuthority: null,
        supply: metadata?.supply || 0,
        circulatingSupply: metadata?.supply || 0,
        lpBurned: false,
        topHolders: [],
        recentTransactions: [],
        lastUpdated: Date.now(),
      };

      this.processNewCoin(memecoin, 'helius');
    } catch (error) {
      console.error('Failed to process Helius token mint:', error);
    }
  }

  // Handle Helius large trade events for whale detection
  private handleHeliusLargeTrade(data: { mint: string; amount: number; price: number; valueUsd: number; buyer: string; timestamp: number }): void {
    const detection = this.newCoins.get(data.mint);
    if (!detection) return;

    // Update whale activity
    const whale = detection.whaleActivity;
    
    if (data.valueUsd >= this.WHALE_THRESHOLD) {
      whale.largeBuys.push({ amount: data.valueUsd, time: data.timestamp });
      whale.totalBuyVolume += data.valueUsd;
      whale.totalInvestedUsd += data.valueUsd;
      whale.whales = new Set(whale.largeBuys.map(b => b.time.toString())).size; // Approximate
      whale.lastUpdated = Date.now();

      // Recalculate alert level
      const totalVolume = whale.largeBuys.reduce((sum, b) => sum + b.amount, 0);
      if (totalVolume >= 50000) whale.whaleAlert = 'extreme';
      else if (totalVolume >= 20000) whale.whaleAlert = 'high';
      else if (totalVolume >= 5000) whale.whaleAlert = 'medium';
      else whale.whaleAlert = 'low';

      // Trigger whale alert (it's always non-'none' here)
      this.onWhaleAlertCallbacks.forEach(cb => {
        try {
          cb(detection);
        } catch (e) {
          console.error('Whale alert callback error:', e);
        }
      });
    }
  }

  // Update age of all coins (call every second)
  updateAges(): void {
    const now = Date.now();
    for (const detection of this.newCoins.values()) {
      detection.ageSeconds = Math.floor((now - detection.detectedAt) / 1000);
    }
  }
}
