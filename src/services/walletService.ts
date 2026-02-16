import { WalletState, TokenBalance } from '../types/trading';
import { useTradingStore } from '../store/tradingStore';
import { Connection, PublicKey } from '@solana/web3.js';

// Get Helius API key from localStorage if available
const getHeliusKey = () => localStorage.getItem('heliusApiKey') || '';

// Use Helius RPC if key available, otherwise public endpoint
const getSolanaRpc = () => {
  const key = getHeliusKey();
  if (key) {
    return `https://mainnet.helius-rpc.com/?api-key=${key}`;
  }
  // Fallback that usually works
  return 'https://solana-api.instantnodes.io';
};

export class WalletService {
  private connection: Connection;
  private connected: boolean = false;
  private address: string | null = null;

  constructor() {
    this.connection = new Connection(getSolanaRpc());
  }

  // Update RPC connection if Helius key changes
  updateRpcConnection(): void {
    this.connection = new Connection(getSolanaRpc());
  }

  async connect(): Promise<boolean> {
    // Check if Phantom or other wallet is installed
    const provider = this.getProvider();
    if (!provider) {
      console.error('No wallet provider found. Please install Phantom or Solflare.');
      return false;
    }

    try {
      // Update RPC connection with latest Helius key before connecting
      this.updateRpcConnection();

      const response = await provider.connect();
      this.address = response.publicKey.toString();
      this.connected = true;

      // Update store
      await this.updateWalletState();

      // Listen for account changes
      provider.on('accountChanged', () => {
        this.updateWalletState();
      });

      return true;
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      return false;
    }
  }

  async disconnect(): Promise<void> {
    const provider = this.getProvider();
    if (provider) {
      await provider.disconnect();
    }
    this.connected = false;
    this.address = null;
    
    useTradingStore.getState().setWallet({
      connected: false,
      address: null,
      balance: 0,
      tokenBalances: [],
    });
  }

  private getProvider(): any {
    if ('phantom' in window) {
      const provider = (window as any).phantom?.solana;
      if (provider?.isPhantom) {
        return provider;
      }
    }
    
    // Check for Solflare
    if ('solflare' in window) {
      const provider = (window as any).solflare;
      if (provider?.isSolflare) {
        return provider;
      }
    }

    return null;
  }

  private async updateWalletState(): Promise<void> {
    if (!this.address) return;

    try {
      // Get SOL balance
      const pubKey = new PublicKey(this.address);
      const balance = await this.connection.getBalance(pubKey);
      const solBalance = balance / 1e9;

      // Get token balances
      const tokenBalances = await this.fetchTokenBalances(this.address);

      const walletState: WalletState = {
        connected: this.connected,
        address: this.address,
        balance: solBalance,
        tokenBalances,
      };

      useTradingStore.getState().setWallet(walletState);
    } catch (error) {
      console.error('Failed to update wallet state:', error);
    }
  }

  private async fetchTokenBalances(address: string): Promise<TokenBalance[]> {
    try {
      const pubKey = new PublicKey(address);
      
      // Get all token accounts
      const tokenAccounts = await this.connection.getParsedTokenAccountsByOwner(
        pubKey,
        { programId: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA') }
      );

      const balances: TokenBalance[] = [];

      for (const account of tokenAccounts.value) {
        const parsed = account.account.data.parsed;
        const mint = parsed.info.mint;
        const amount = Number(parsed.info.tokenAmount.uiAmount);

        if (amount > 0) {
          // Fetch token metadata
          const metadata = await this.fetchTokenMetadata(mint);
          
          // Calculate USD value (this would need price data)
          const valueUsd = 0; // Will be updated with price data

          balances.push({
            mint,
            symbol: metadata?.symbol || 'UNKNOWN',
            balance: amount,
            valueUsd,
          });
        }
      }

      return balances;
    } catch (error) {
      console.error('Failed to fetch token balances:', error);
      return [];
    }
  }

  private async fetchTokenMetadata(mint: string): Promise<{ symbol: string; name: string } | null> {
    try {
      const response = await fetch(`https://token-list-api.solana.cloud/v1/tokens?address=${mint}`);
      const data = await response.json();
      if (data && data.length > 0) {
        return {
          symbol: data[0].symbol,
          name: data[0].name,
        };
      }
    } catch {
      // Ignore errors
    }
    return null;
  }

  getAddress(): string | null {
    return this.address;
  }

  isConnected(): boolean {
    return this.connected;
  }

  async signTransaction(transaction: string): Promise<string | null> {
    const provider = this.getProvider();
    if (!provider || !this.connected) {
      console.error('Wallet not connected');
      return null;
    }

    try {
      // Sign with wallet
      const signed = await provider.signTransaction(transaction);
      
      // Send transaction
      const signature = await this.connection.sendRawTransaction(signed.serialize());
      
      // Wait for confirmation
      await this.connection.confirmTransaction(signature, 'confirmed');
      
      return signature;
    } catch (error) {
      console.error('Transaction failed:', error);
      return null;
    }
  }
}

export const walletService = new WalletService();
