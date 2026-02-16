import { createClient, SupabaseClient } from '@supabase/supabase-js';
import { Memecoin } from '../types/trading';

// You'll need to create a Supabase project and get these credentials
const SUPABASE_URL = (import.meta as any).env?.VITE_SUPABASE_URL || '';
const SUPABASE_KEY = (import.meta as any).env?.VITE_SUPABASE_KEY || '';

export interface DetectedCoinRecord {
  id?: string;
  token_address: string;
  symbol: string;
  name: string;
  detected_at: string;
  source: 'dexscreener' | 'helius' | 'pumpfun' | 'jupiter';
  signal: 'green' | 'yellow' | 'red';
  confidence: number;
  liquidity: number;
  market_cap: number;
  volume_24h: number;
  ai_score: number;
  rug_pull_risk: number;
  did_pump?: boolean;
  pump_percent?: number;
  did_rug?: boolean;
  notes?: string;
}

export interface TradeRecord {
  id?: string;
  token_address: string;
  symbol: string;
  type: 'buy' | 'sell';
  amount_sol: number;
  price_usd: number;
  quantity: number;
  pnl_sol?: number;
  pnl_percent?: number;
  timestamp: string;
  tx_signature?: string;
  is_paper_trade: boolean;
  ai_signal?: string;
  confidence?: number;
  exit_reason?: string;
}

export class DatabaseService {
  private client: SupabaseClient | null = null;
  private enabled: boolean = false;

  constructor() {
    if (SUPABASE_URL && SUPABASE_KEY) {
      this.client = createClient(SUPABASE_URL, SUPABASE_KEY);
      this.enabled = true;
    } else {
      console.log('Supabase not configured - using localStorage fallback');
    }
  }

  isEnabled(): boolean {
    return this.enabled;
  }

  // Store detected coin
  async saveDetectedCoin(coin: Memecoin, source: string, signal: any): Promise<void> {
    const record: DetectedCoinRecord = {
      token_address: coin.address,
      symbol: coin.symbol,
      name: coin.name,
      detected_at: new Date().toISOString(),
      source: source as any,
      signal: signal.signal,
      confidence: signal.confidence,
      liquidity: coin.liquidity,
      market_cap: coin.marketCap,
      volume_24h: coin.volume24h,
      ai_score: signal.score,
      rug_pull_risk: 0,
    };

    if (this.enabled && this.client) {
      const { error } = await this.client
        .from('detected_coins')
        .upsert(record, { onConflict: 'token_address' });

      if (error) {
        console.error('Failed to save to Supabase:', error);
        this.fallbackSave('detected_coins', record);
      }
    } else {
      this.fallbackSave('detected_coins', record);
    }
  }

  // Get all detected coins
  async getDetectedCoins(): Promise<DetectedCoinRecord[]> {
    if (this.enabled && this.client) {
      const { data, error } = await this.client
        .from('detected_coins')
        .select('*')
        .order('detected_at', { ascending: false })
        .limit(100);

      if (error) {
        console.error('Failed to fetch from Supabase:', error);
        return this.fallbackGet('detected_coins');
      }

      return data || [];
    }

    return this.fallbackGet('detected_coins');
  }

  // Record a trade
  async recordTrade(trade: TradeRecord): Promise<void> {
    if (this.enabled && this.client) {
      const { error } = await this.client
        .from('trades')
        .insert(trade);

      if (error) {
        console.error('Failed to record trade:', error);
        this.fallbackSave('trades', trade);
      }
    } else {
      this.fallbackSave('trades', trade);
    }
  }

  // Get trade history
  async getTradeHistory(): Promise<TradeRecord[]> {
    if (this.enabled && this.client) {
      const { data, error } = await this.client
        .from('trades')
        .select('*')
        .order('timestamp', { ascending: false })
        .limit(200);

      if (error) {
        console.error('Failed to fetch trades:', error);
        return this.fallbackGet('trades');
      }

      return data || [];
    }

    return this.fallbackGet('trades');
  }

  // Get AI training data (successful patterns)
  async getSuccessfulPatterns(): Promise<DetectedCoinRecord[]> {
    if (this.enabled && this.client) {
      const { data, error } = await this.client
        .from('detected_coins')
        .select('*')
        .eq('did_pump', true)
        .gt('pump_percent', 50)
        .order('pump_percent', { ascending: false })
        .limit(50);

      if (error) {
        console.error('Failed to fetch patterns:', error);
        return [];
      }

      return data || [];
    }

    const coins = this.fallbackGet('detected_coins');
    return coins.filter((c: any) => c.did_pump && c.pump_percent > 50);
  }

  // Update coin outcome (did it pump or rug?)
  async updateCoinOutcome(tokenAddress: string, outcome: { didPump: boolean; pumpPercent?: number; didRug?: boolean }): Promise<void> {
    if (this.enabled && this.client) {
      const { error } = await this.client
        .from('detected_coins')
        .update({
          did_pump: outcome.didPump,
          pump_percent: outcome.pumpPercent,
          did_rug: outcome.didRug,
        })
        .eq('token_address', tokenAddress);

      if (error) {
        console.error('Failed to update outcome:', error);
      }
    }

    const coins = this.fallbackGet('detected_coins');
    const index = coins.findIndex((c: any) => c.token_address === tokenAddress);
    if (index !== -1) {
      coins[index] = {
        ...coins[index],
        did_pump: outcome.didPump,
        pump_percent: outcome.pumpPercent,
        did_rug: outcome.didRug,
      };
      localStorage.setItem('db_detected_coins', JSON.stringify(coins));
    }
  }

  // Get performance stats
  async getPerformanceStats(): Promise<{
    totalCoins: number;
    pumped: number;
    rugged: number;
    winRate: number;
    avgPump: number;
  }> {
    const coins = await this.getDetectedCoins();
    const pumped = coins.filter(c => c.did_pump).length;
    const rugged = coins.filter(c => c.did_rug).length;
    
    const pumps = coins.filter(c => c.did_pump && c.pump_percent);
    const avgPump = pumps.length > 0 
      ? pumps.reduce((sum, c) => sum + (c.pump_percent || 0), 0) / pumps.length 
      : 0;

    return {
      totalCoins: coins.length,
      pumped,
      rugged,
      winRate: coins.length > 0 ? (pumped / coins.length) * 100 : 0,
      avgPump,
    };
  }

  // Private fallback methods using localStorage
  private fallbackSave(key: string, data: any): void {
    const existing = JSON.parse(localStorage.getItem(`db_${key}`) || '[]');
    
    if (data.token_address) {
      const index = existing.findIndex((item: any) => item.token_address === data.token_address);
      if (index !== -1) {
        existing[index] = { ...existing[index], ...data };
      } else {
        existing.push({ ...data, id: crypto.randomUUID() });
      }
    } else {
      existing.push({ ...data, id: crypto.randomUUID() });
    }

    if (existing.length > 500) {
      existing.splice(0, existing.length - 500);
    }

    localStorage.setItem(`db_${key}`, JSON.stringify(existing));
  }

  private fallbackGet(key: string): any[] {
    return JSON.parse(localStorage.getItem(`db_${key}`) || '[]');
  }
}

export const databaseService = new DatabaseService();
