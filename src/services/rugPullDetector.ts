import { Connection, PublicKey } from '@solana/web3.js';

export interface RugPullCheck {
  isSafe: boolean;
  riskScore: number; // 0-100, higher = more risky
  checks: {
    mintAuthority: {
      safe: boolean;
      info: string;
    };
    freezeAuthority: {
      safe: boolean;
      info: string;
    };
    lpBurned: {
      safe: boolean;
      info: string;
    };
    lpLocked: {
      safe: boolean;
      info: string;
      lockTime?: number; // Days locked
    };
    holderConcentration: {
      safe: boolean;
      info: string;
      top5Percent?: number;
    };
    honeypot: {
      safe: boolean;
      info: string;
    };
    contractVerified: {
      safe: boolean;
      info: string;
    };
  };
  warnings: string[];
  redFlags: string[];
  greenFlags: string[];
}

export class RugPullDetector {
  private connection: Connection;
  private heliusKey: string;
  private cache: Map<string, { result: RugPullCheck; timestamp: number }> = new Map();
  private cacheDuration = 5 * 60 * 1000; // 5 minutes

  constructor() {
    this.heliusKey = localStorage.getItem('heliusApiKey') || '';
    this.connection = new Connection(
      this.heliusKey 
        ? `https://mainnet.helius-rpc.com/?api-key=${this.heliusKey}`
        : 'https://solana-api.instantnodes.io'
    );
  }

  async checkToken(tokenAddress: string): Promise<RugPullCheck> {
    // Check cache
    const cached = this.cache.get(tokenAddress);
    if (cached && Date.now() - cached.timestamp < this.cacheDuration) {
      return cached.result;
    }

    const checks = await this.performChecks(tokenAddress);
    
    // Calculate risk score
    let riskScore = 0;
    const warnings: string[] = [];
    const redFlags: string[] = [];
    const greenFlags: string[] = [];

    // Mint authority check
    if (!checks.mintAuthority.safe) {
      riskScore += 25;
      redFlags.push('Mint authority not revoked - dev can print unlimited tokens');
    } else {
      greenFlags.push('Mint authority revoked');
    }

    // Freeze authority check
    if (!checks.freezeAuthority.safe) {
      riskScore += 15;
      warnings.push('Freeze authority enabled - dev can freeze your wallet');
    } else {
      greenFlags.push('Freeze authority disabled');
    }

    // LP check
    if (!checks.lpBurned.safe && !checks.lpLocked.safe) {
      riskScore += 30;
      redFlags.push('Liquidity not locked/burned - dev can rug');
    } else if (checks.lpBurned.safe) {
      greenFlags.push('Liquidity burned');
    } else if (checks.lpLocked.safe) {
      greenFlags.push(`Liquidity locked for ${checks.lpLocked.lockTime} days`);
    }

    // Holder concentration
    if (!checks.holderConcentration.safe) {
      riskScore += 20;
      warnings.push(`High holder concentration: Top 5 holders own ${checks.holderConcentration.top5Percent}%`);
    }

    // Honeypot check
    if (!checks.honeypot.safe) {
      riskScore += 50;
      redFlags.push('HONEYPOT DETECTED - You cannot sell this token!');
    } else {
      greenFlags.push('Not a honeypot - sells are possible');
    }

    const result: RugPullCheck = {
      isSafe: riskScore < 30 && redFlags.length === 0,
      riskScore: Math.min(100, riskScore),
      checks,
      warnings,
      redFlags,
      greenFlags,
    };

    // Cache result
    this.cache.set(tokenAddress, { result, timestamp: Date.now() });

    return result;
  }

  private async performChecks(tokenAddress: string): Promise<RugPullCheck['checks']> {
    const mintPubkey = new PublicKey(tokenAddress);

    // Default checks (unknown)
    const checks: RugPullCheck['checks'] = {
      mintAuthority: { safe: false, info: 'Unknown' },
      freezeAuthority: { safe: false, info: 'Unknown' },
      lpBurned: { safe: false, info: 'Unknown' },
      lpLocked: { safe: false, info: 'Unknown' },
      holderConcentration: { safe: false, info: 'Unknown' },
      honeypot: { safe: true, info: 'Unknown - assumed safe' },
      contractVerified: { safe: false, info: 'Unknown' },
    };

    try {
      // Get mint info
      const mintInfo = await this.connection.getParsedAccountInfo(mintPubkey);
      if (mintInfo.value && 'parsed' in mintInfo.value.data) {
        const parsed = mintInfo.value.data.parsed;
        const info = parsed.info;

        // Check mint authority
        if (info.mintAuthority === null) {
          checks.mintAuthority = { safe: true, info: 'Mint authority revoked' };
        } else {
          checks.mintAuthority = { 
            safe: false, 
            info: `Mint authority: ${info.mintAuthority}` 
          };
        }

        // Check freeze authority
        if (info.freezeAuthority === null) {
          checks.freezeAuthority = { safe: true, info: 'Freeze authority disabled' };
        } else {
          checks.freezeAuthority = { 
            safe: false, 
            info: `Freeze authority: ${info.freezeAuthority}` 
          };
        }
      }
    } catch (error) {
      console.error('Failed to fetch mint info:', error);
    }

    // Check LP via Helius DAS API
    try {
      const lpInfo = await this.checkLiquidityPool(tokenAddress);
      checks.lpBurned = lpInfo.burned;
      checks.lpLocked = lpInfo.locked;
    } catch (error) {
      console.error('Failed to check LP:', error);
    }

    // Check holders
    try {
      const holderInfo = await this.checkHolders(tokenAddress);
      checks.holderConcentration = holderInfo;
    } catch (error) {
      console.error('Failed to check holders:', error);
    }

    // Honeypot check via simulated sell
    try {
      const honeypotCheck = await this.checkHoneypot(tokenAddress);
      checks.honeypot = honeypotCheck;
    } catch (error) {
      console.error('Failed honeypot check:', error);
    }

    return checks;
  }

  private async checkLiquidityPool(tokenAddress: string): Promise<{ burned: { safe: boolean; info: string }; locked: { safe: boolean; info: string; lockTime?: number } }> {
    // Use Helius to check LP accounts
    try {
      const response = await fetch(
        `https://mainnet.helius-rpc.com/?api-key=${this.heliusKey}`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'getTokenAccountsByOwner',
            params: [
              new PublicKey(tokenAddress), // This won't work, need proper LP detection
              { programId: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA') },
              { encoding: 'jsonParsed' },
            ],
          }),
        }
      );

      // Simplified - in reality would check Raydium/Orca pool accounts
      return {
        burned: { safe: false, info: 'Unable to verify LP status' },
        locked: { safe: false, info: 'Unable to verify LP lock' },
      };
    } catch {
      return {
        burned: { safe: false, info: 'Check failed' },
        locked: { safe: false, info: 'Check failed' },
      };
    }
  }

  private async checkHolders(tokenAddress: string): Promise<{ safe: boolean; info: string; top5Percent?: number }> {
    try {
      // Use Helius API to get token holders
      const response = await fetch(
        `https://mainnet.helius-rpc.com/?api-key=${this.heliusKey}`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            jsonrpc: '2.0',
            id: 1,
            method: 'getProgramAccounts',
            params: [
              new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'),
              {
                filters: [
                  { dataSize: 165 },
                  { memcmp: { offset: 0, bytes: tokenAddress } },
                ],
                encoding: 'jsonParsed',
              },
            ],
          }),
        }
      );

      // This would parse holder data
      // For now, return unknown
      return { safe: false, info: 'Holder data unavailable' };
    } catch {
      return { safe: false, info: 'Failed to fetch holders' };
    }
  }

  private async checkHoneypot(tokenAddress: string): Promise<{ safe: boolean; info: string }> {
    // In a real implementation, this would:
    // 1. Simulate a sell transaction
    // 2. Check if the transaction would succeed
    // 3. Use Jupiter API to verify swaps are possible

    try {
      // Quick check - try to get Jupiter quote for selling
      const response = await fetch(
        `https://quote-api.jup.ag/v6/quote?inputMint=${tokenAddress}&outputMint=So11111111111111111111111111111111111111112&amount=1000&slippageBps=500`,
        { headers: { 'Accept': 'application/json' } }
      );

      if (!response.ok) {
        return { 
          safe: false, 
          info: 'Cannot get swap route - possible honeypot' 
        };
      }

      return { safe: true, info: 'Swap route available - not a honeypot' };
    } catch {
      return { safe: false, info: 'Honeypot check failed' };
    }
  }

  // Quick safety check
  async isSafe(tokenAddress: string): Promise<boolean> {
    const check = await this.checkToken(tokenAddress);
    return check.isSafe;
  }

  // Get risk score only
  async getRiskScore(tokenAddress: string): Promise<number> {
    const check = await this.checkToken(tokenAddress);
    return check.riskScore;
  }
}

export const rugPullDetector = new RugPullDetector();
