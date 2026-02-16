import { Connection, PublicKey, VersionedTransaction } from '@solana/web3.js';

export interface SwapParams {
  inputMint: string;
  outputMint: string;
  amount: number; // in lamports/smallest unit
  slippageBps: number; // e.g., 50 = 0.5%
}

export interface SwapResult {
  signature: string | null;
  success: boolean;
  error?: string;
  inputAmount: number;
  outputAmount: number;
  price: number;
  route: any;
}

export class JupiterSwapService {
  private connection: Connection;
  private wallet: any; // Wallet adapter
  private heliusKey: string;

  constructor(wallet: any) {
    this.wallet = wallet;
    this.heliusKey = localStorage.getItem('heliusApiKey') || '';
    this.connection = new Connection(
      this.heliusKey 
        ? `https://mainnet.helius-rpc.com/?api-key=${this.heliusKey}`
        : 'https://solana-api.instantnodes.io'
    );
  }

  async getSwapRoutes(params: SwapParams): Promise<any[]> {
    try {
      // Validate addresses
      new PublicKey(params.inputMint);
      new PublicKey(params.outputMint);
      
      // Use Jupiter API to get routes
      const response = await fetch(
        `https://quote-api.jup.ag/v6/quote?inputMint=${params.inputMint}&outputMint=${params.outputMint}&amount=${params.amount}&slippageBps=${params.slippageBps}`,
        {
          headers: {
            'Accept': 'application/json',
          },
        }
      );

      if (!response.ok) {
        throw new Error(`Jupiter API error: ${response.status}`);
      }

      const data = await response.json();
      return data.data || [];
    } catch (error) {
      console.error('Failed to get swap routes:', error);
      return [];
    }
  }

  async executeSwap(params: SwapParams): Promise<SwapResult> {
    if (!this.wallet || !this.wallet.publicKey) {
      return {
        signature: null,
        success: false,
        error: 'Wallet not connected',
        inputAmount: params.amount,
        outputAmount: 0,
        price: 0,
        route: null,
      };
    }

    try {
      // Get best route
      const routes = await this.getSwapRoutes(params);
      if (routes.length === 0) {
        throw new Error('No swap routes available');
      }

      const bestRoute = routes[0];

      // Get swap transaction from Jupiter
      const swapResponse = await fetch('https://quote-api.jup.ag/v6/swap', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          route: bestRoute,
          userPublicKey: this.wallet.publicKey.toString(),
          wrapAndUnwrapSol: true,
          computeUnitPriceMicroLamports: 50000, // Priority fee for faster execution
        }),
      });

      if (!swapResponse.ok) {
        throw new Error(`Swap preparation failed: ${swapResponse.status}`);
      }

      const swapData = await swapResponse.json();
      const swapTransaction = swapData.swapTransaction;

      if (!swapTransaction) {
        throw new Error('No swap transaction returned');
      }

      // Deserialize and sign transaction
      const transaction = VersionedTransaction.deserialize(
        Buffer.from(swapTransaction, 'base64')
      );

      // Sign with wallet
      const signed = await this.wallet.signTransaction(transaction);

      // Send transaction
      const signature = await this.connection.sendRawTransaction(signed.serialize(), {
        maxRetries: 3,
        skipPreflight: false,
        preflightCommitment: 'confirmed',
      });

      // Wait for confirmation
      const confirmation = await this.connection.confirmTransaction(signature, 'confirmed');

      if (confirmation.value.err) {
        throw new Error(`Transaction failed: ${confirmation.value.err}`);
      }

      return {
        signature,
        success: true,
        inputAmount: params.amount,
        outputAmount: bestRoute.outAmount,
        price: bestRoute.price,
        route: bestRoute,
      };
    } catch (error: any) {
      console.error('Swap execution failed:', error);
      return {
        signature: null,
        success: false,
        error: error.message || 'Swap failed',
        inputAmount: params.amount,
        outputAmount: 0,
        price: 0,
        route: null,
      };
    }
  }

  // Quick buy function for memecoins
  async quickBuy(tokenAddress: string, solAmount: number, slippageBps: number = 100): Promise<SwapResult> {
    const WSOL_MINT = 'So11111111111111111111111111111111111111112';
    const lamports = Math.floor(solAmount * 1e9); // Convert SOL to lamports

    return this.executeSwap({
      inputMint: WSOL_MINT,
      outputMint: tokenAddress,
      amount: lamports,
      slippageBps,
    });
  }

  // Quick sell function
  async quickSell(tokenAddress: string, tokenAmount: number, decimals: number = 9, slippageBps: number = 100): Promise<SwapResult> {
    const WSOL_MINT = 'So11111111111111111111111111111111111111112';
    const atomicAmount = Math.floor(tokenAmount * Math.pow(10, decimals));

    return this.executeSwap({
      inputMint: tokenAddress,
      outputMint: WSOL_MINT,
      amount: atomicAmount,
      slippageBps,
    });
  }

  // Get token price in USD
  async getTokenPrice(tokenAddress: string): Promise<number> {
    try {
      const USDC_MINT = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
      const routes = await this.getSwapRoutes({
        inputMint: tokenAddress,
        outputMint: USDC_MINT,
        amount: 1000000000, // 1 token (assuming 9 decimals)
        slippageBps: 100,
      });

      if (routes.length === 0) return 0;
      return routes[0].outAmount / 1000000; // Convert from USDC decimals (6)
    } catch (error) {
      console.error('Failed to get token price:', error);
      return 0;
    }
  }

  // Check if token has liquidity on Jupiter
  async hasLiquidity(tokenAddress: string): Promise<boolean> {
    try {
      const WSOL_MINT = 'So11111111111111111111111111111111111111112';
      const routes = await this.getSwapRoutes({
        inputMint: tokenAddress,
        outputMint: WSOL_MINT,
        amount: 1000000, // Small amount
        slippageBps: 500, // 5% slippage
      });

      return routes.length > 0 && routes[0].otherAmountThreshold > 0;
    } catch {
      return false;
    }
  }
}

export const jupiterSwapService = new JupiterSwapService(null);
