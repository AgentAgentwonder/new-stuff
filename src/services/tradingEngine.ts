import { TradeExecution } from '../types/trading';
import { useTradingStore } from '../store/tradingStore';

// Jupiter Swap API
const JUPITER_QUOTE_API = 'https://quote-api.jup.ag/v6';
const JUPITER_SWAP_API = 'https://quote-api.jup.ag/v6/swap';

export class TradingEngine {
  private wallet: string | null = null;

  setWallet(address: string): void {
    this.wallet = address;
  }

  async executeBuy(
    coinAddress: string,
    amountUsd: number,
    maxSlippage: number = 2.5
  ): Promise<TradeExecution | null> {
    if (!this.wallet) {
      console.error('Wallet not connected');
      return null;
    }

    const tradeId = `buy-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    const execution: TradeExecution = {
      id: tradeId,
      coinAddress,
      type: 'buy',
      amount: 0, // Will be set after quote
      priceUsd: 0,
      totalUsd: amountUsd,
      slippage: maxSlippage,
      status: 'pending',
      timestamp: Date.now(),
    };

    useTradingStore.getState().addTrade(execution);

    try {
      // Step 1: Get quote
      const quoteResponse = await fetch(
        `${JUPITER_QUOTE_API}/quote?inputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&outputMint=${coinAddress}&amount=${Math.floor(amountUsd * 1000000)}&slippageBps=${maxSlippage * 100}&onlyDirectRoutes=false`
      );

      if (!quoteResponse.ok) {
        throw new Error('Failed to get quote');
      }

      const quote = await quoteResponse.json();

      // Update execution with quote data
      execution.amount = Number(quote.outAmount) / Math.pow(10, quote.outputMintDecimals || 9);
      execution.priceUsd = amountUsd / execution.amount;

      // Step 2: Get swap transaction
      const swapResponse = await fetch(`${JUPITER_SWAP_API}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          quoteResponse: quote,
          userPublicKey: this.wallet,
          wrapAndUnwrapSol: true,
          prioritizationFeeLamports: 10000, // 0.001 SOL priority fee for faster execution
        }),
      });

      if (!swapResponse.ok) {
        throw new Error('Failed to get swap transaction');
      }

      const swapData = await swapResponse.json();

      // Step 3: Sign and send transaction via wallet adapter
      // This requires the wallet to be connected via the UI
      const signature = await this.signAndSend(swapData.swapTransaction);

      if (signature) {
        useTradingStore.getState().updateTradeStatus(tradeId, 'confirmed', signature);
        execution.status = 'confirmed';
        execution.signature = signature;
      } else {
        throw new Error('Transaction failed');
      }

      return execution;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      useTradingStore.getState().updateTradeStatus(tradeId, 'failed', undefined, errorMsg);
      execution.status = 'failed';
      execution.error = errorMsg;
      return execution;
    }
  }

  async executeSell(
    coinAddress: string,
    amount: number,
    maxSlippage: number = 2.5
  ): Promise<TradeExecution | null> {
    if (!this.wallet) {
      console.error('Wallet not connected');
      return null;
    }

    const tradeId = `sell-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    const execution: TradeExecution = {
      id: tradeId,
      coinAddress,
      type: 'sell',
      amount,
      priceUsd: 0, // Will be set after quote
      totalUsd: 0,
      slippage: maxSlippage,
      status: 'pending',
      timestamp: Date.now(),
    };

    useTradingStore.getState().addTrade(execution);

    try {
      // Get token decimals first
      const tokenInfo = await this.getTokenInfo(coinAddress);
      const rawAmount = Math.floor(amount * Math.pow(10, tokenInfo.decimals || 9));

      // Step 1: Get quote
      const quoteResponse = await fetch(
        `${JUPITER_QUOTE_API}/quote?inputMint=${coinAddress}&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=${rawAmount}&slippageBps=${maxSlippage * 100}&onlyDirectRoutes=false`
      );

      if (!quoteResponse.ok) {
        throw new Error('Failed to get quote');
      }

      const quote = await quoteResponse.json();

      // Update execution with quote data
      execution.totalUsd = Number(quote.outAmount) / 1000000; // USDC has 6 decimals
      execution.priceUsd = execution.totalUsd / amount;

      // Step 2: Get swap transaction
      const swapResponse = await fetch(`${JUPITER_SWAP_API}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          quoteResponse: quote,
          userPublicKey: this.wallet,
          wrapAndUnwrapSol: true,
          prioritizationFeeLamports: 10000,
        }),
      });

      if (!swapResponse.ok) {
        throw new Error('Failed to get swap transaction');
      }

      const swapData = await swapResponse.json();

      // Step 3: Sign and send
      const signature = await this.signAndSend(swapData.swapTransaction);

      if (signature) {
        useTradingStore.getState().updateTradeStatus(tradeId, 'confirmed', signature);
        execution.status = 'confirmed';
        execution.signature = signature;
      } else {
        throw new Error('Transaction failed');
      }

      return execution;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Unknown error';
      useTradingStore.getState().updateTradeStatus(tradeId, 'failed', undefined, errorMsg);
      execution.status = 'failed';
      execution.error = errorMsg;
      return execution;
    }
  }

  private async getTokenInfo(mintAddress: string): Promise<{ decimals: number }> {
    try {
      // Use Jupiter token list or Solana RPC
      const response = await fetch(`https://token-list-api.solana.cloud/v1/tokens?address=${mintAddress}`);
      const data = await response.json();
      return { decimals: data?.[0]?.decimals || 9 };
    } catch {
      return { decimals: 9 };
    }
  }

  private async signAndSend(transaction: string): Promise<string | null> {
    // This will be implemented with wallet adapter
    // For now, return null - the actual signing happens in the UI layer
    console.log('Transaction ready for signing:', transaction.substring(0, 50) + '...');
    return null;
  }

  // Calculate optimal slippage based on market conditions
  calculateOptimalSlippage(liquidity: number, volume: number): number {
    const baseSlippage = 0.5;
    const liquidityFactor = Math.max(0, 5 - liquidity / 10000);
    const volumeFactor = Math.max(0, volume / 100000);
    return Math.min(baseSlippage + liquidityFactor + volumeFactor, 5);
  }
}

export const tradingEngine = new TradingEngine();
