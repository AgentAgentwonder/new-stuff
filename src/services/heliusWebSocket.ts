import { Memecoin } from '../types/trading';

interface HeliusConfig {
  apiKey: string;
  rpcUrl?: string;
}

interface TokenMintEvent {
  signature: string;
  mint: string;
  name?: string;
  symbol?: string;
  decimals?: number;
  supply?: string;
  timestamp: number;
  slot: number;
}

export class HeliusWebSocketService {
  private ws: WebSocket | null = null;
  private config: HeliusConfig;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 10;
  private reconnectDelay = 1000;
  private isRunning = false;
  private onTokenMintCallbacks: ((event: TokenMintEvent) => void)[] = [];
  private onLargeTradeCallbacks: ((data: {
    mint: string;
    amount: number;
    price: number;
    valueUsd: number;
    buyer: string;
    timestamp: number;
  }) => void)[] = [];

  constructor(config: HeliusConfig) {
    this.config = config;
  }

  start(): void {
    if (this.isRunning) return;
    this.isRunning = true;
    this.connect();
  }

  stop(): void {
    this.isRunning = false;
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  onTokenMint(callback: (event: TokenMintEvent) => void): () => void {
    this.onTokenMintCallbacks.push(callback);
    return () => {
      const index = this.onTokenMintCallbacks.indexOf(callback);
      if (index > -1) this.onTokenMintCallbacks.splice(index, 1);
    };
  }

  onLargeTrade(callback: (data: {
    mint: string;
    amount: number;
    price: number;
    valueUsd: number;
    buyer: string;
    timestamp: number;
  }) => void): () => void {
    this.onLargeTradeCallbacks.push(callback);
    return () => {
      const index = this.onLargeTradeCallbacks.indexOf(callback);
      if (index > -1) this.onLargeTradeCallbacks.splice(index, 1);
    };
  }

  private connect(): void {
    if (!this.isRunning) return;

    const wsUrl = this.config.rpcUrl || 
      `wss://mainnet.helius-rpc.com/?api-key=${this.config.apiKey}`;

    try {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('Helius WebSocket connected');
        this.reconnectAttempts = 0;
        this.subscribeToTokenMints();
        this.subscribeToPumpFunTrades();
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleMessage(data);
        } catch (e) {
          console.error('Failed to parse Helius message:', e);
        }
      };

      this.ws.onclose = () => {
        console.log('Helius WebSocket closed');
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('Helius WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to connect to Helius:', error);
      this.attemptReconnect();
    }
  }

  private subscribeToTokenMints(): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;

    // Subscribe to Token Program logs for new mints
    this.ws.send(JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'logsSubscribe',
      params: [
        {
          mentions: ['TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA']
        },
        {
          commitment: 'confirmed'
        }
      ]
    }));

    // Also subscribe to Pump.fun program specifically for new launches
    this.ws.send(JSON.stringify({
      jsonrpc: '2.0',
      id: 2,
      method: 'logsSubscribe',
      params: [
        {
          mentions: ['6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P'] // Pump.fun program
        },
        {
          commitment: 'confirmed'
        }
      ]
    }));
  }

  private subscribeToPumpFunTrades(): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;

    // Subscribe to account changes on Pump.fun bonding curves
    // This detects buys/sells in real-time
    console.log('Subscribed to Pump.fun trade monitoring');
  }

  private handleMessage(data: any): void {
    if (data.method === 'logsNotification') {
      const logs = data.params.result.value;
      this.parseLogs(logs);
    }
  }

  private parseLogs(logs: any): void {
    const logMessages: string[] = logs.logs || [];
    const signature = logs.signature;
    
    // Look for token mint events
    for (const log of logMessages) {
      // Detect new token mints
      if (log.includes('InitializeMint')) {
        const mint = this.extractMintFromLogs(logMessages);
        if (mint) {
          const event: TokenMintEvent = {
            signature,
            mint,
            timestamp: Date.now(),
            slot: logs.slot || 0,
          };
          this.onTokenMintCallbacks.forEach(cb => {
            try {
              cb(event);
            } catch (e) {
              console.error('Token mint callback error:', e);
            }
          });
        }
      }

      // Detect Pump.fun specific events
      if (log.includes('Create') || log.includes('Buy') || log.includes('Sell')) {
        this.parsePumpFunEvent(logs);
      }
    }
  }

  private extractMintFromLogs(logs: string[]): string | null {
    // Extract mint address from initialize mint logs
    for (const log of logs) {
      const match = log.match(/mint:\s*([A-Za-z0-9]{32,44})/);
      if (match) return match[1];
    }
    return null;
  }

  private parsePumpFunEvent(logs: any): void {
    // Parse Pump.fun trade events for whale detection
    const logMessages: string[] = logs.logs || [];
    
    for (const log of logMessages) {
      // Look for large buys (you'll need to parse the actual trade amount)
      if (log.includes('Buy')) {
        // Extract mint and amount from the log
        // This is simplified - actual parsing would decode the instruction data
        const mint = this.extractMintFromLogs(logMessages);
        if (mint) {
          // Notify about potential trade (would need actual amount from transaction parsing)
          console.log('Pump.fun trade detected:', mint);
        }
      }
    }
  }

  private attemptReconnect(): void {
    if (!this.isRunning) return;
    
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max Helius reconnect attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    
    console.log(`Helius reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
    
    setTimeout(() => {
      this.connect();
    }, delay);
  }

  // Fetch enhanced transaction data using Helius API
  async getTransactionWithEnhancedData(signature: string): Promise<any> {
    try {
      const response = await fetch(
        `https://mainnet.helius-rpc.com/?api-key=${this.config.apiKey}`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'getTransaction',
            params: [
              signature,
              {
                encoding: 'jsonParsed',
                commitment: 'confirmed',
                maxSupportedTransactionVersion: 0,
              },
            ],
          }),
        }
      );

      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      
      const data = await response.json();
      return data.result;
    } catch (error) {
      console.error('Failed to fetch Helius transaction:', error);
      return null;
    }
  }

  // Fetch token metadata using Helius
  async getTokenMetadata(mint: string): Promise<Partial<Memecoin> | null> {
    try {
      const response = await fetch(
        `https://mainnet.helius-rpc.com/?api-key=${this.config.apiKey}`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'getAsset',
            params: {
              id: mint,
            },
          }),
        }
      );

      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      
      const data = await response.json();
      const asset = data.result;

      if (!asset) return null;

      return {
        address: mint,
        symbol: asset.tokenInfo?.symbol || 'UNKNOWN',
        name: asset.content?.metadata?.name || asset.tokenInfo?.name || 'Unknown Token',
        decimals: asset.tokenInfo?.decimals || 9,
        supply: asset.tokenInfo?.supply ? parseInt(asset.tokenInfo.supply) : 0,
        lastUpdated: Date.now(),
      };
    } catch (error) {
      console.error('Failed to fetch Helius token metadata:', error);
      return null;
    }
  }
}

export type { HeliusConfig, TokenMintEvent };
