import { Memecoin, TradeSignal, RiskConfig } from '../types/trading';

export class RiskCalculator {
  private config: RiskConfig;

  constructor(config: RiskConfig) {
    this.config = config;
  }

  updateConfig(config: RiskConfig): void {
    this.config = config;
  }

  calculateRiskScore(coin: Memecoin): number {
    let score = 0;
    const maxScore = 100;

    // Liquidity check (0-25 points)
    const liquidityScore = Math.min((coin.liquidity / this.config.minLiquidity) * 25, 25);
    score += liquidityScore;

    // Holder count check (0-20 points)
    const holderScore = Math.min((coin.holderCount / this.config.minHolderCount) * 20, 20);
    score += holderScore;

    // LP burned check (15 points)
    if (coin.lpBurned) score += 15;

    // Mint authority check (10 points - null is better)
    if (!coin.mintAuthority) score += 10;

    // Freeze authority check (10 points - null is better)
    if (!coin.freezeAuthority) score += 10;

    // Price stability check (0-10 points based on 24h change)
    const priceStability = Math.max(0, 10 - Math.abs(coin.priceChange24h) / 10);
    score += priceStability;

    return Math.min(score, maxScore);
  }

  calculateConfidence(coin: Memecoin, riskScore: number): number {
    // Base confidence from risk score
    let confidence = riskScore * 0.6;

    // Volume indicator (0-20 points)
    const volumeScore = Math.min((coin.volume24h / 50000) * 20, 20);
    confidence += volumeScore;

    // Market cap stability (0-20 points)
    if (coin.marketCap > 100000 && coin.marketCap < 10000000) {
      confidence += 20; // Sweet spot for memecoins
    } else if (coin.marketCap >= 10000000) {
      confidence += 10; // Already pumped
    } else {
      confidence += Math.min((coin.marketCap / 100000) * 20, 20);
    }

    return Math.min(confidence, 100);
  }

  generateSignal(coin: Memecoin): TradeSignal {
    const riskScore = this.calculateRiskScore(coin);
    const confidence = this.calculateConfidence(coin, riskScore);

    let signal: 'green' | 'yellow' | 'red';
    const reasons: string[] = [];

    // Determine signal
    if (confidence >= this.config.greenThreshold && riskScore >= 70) {
      signal = 'green';
    } else if (confidence >= this.config.yellowThreshold && riskScore >= 50) {
      signal = 'yellow';
    } else {
      signal = 'red';
    }

    // Generate reasons
    if (coin.liquidity < this.config.minLiquidity) {
      reasons.push(`Low liquidity ($${coin.liquidity.toLocaleString()})`);
    }
    if (coin.holderCount < this.config.minHolderCount) {
      reasons.push(`Low holder count (${coin.holderCount})`);
    }
    if (!coin.lpBurned) {
      reasons.push('LP not burned - rug risk');
    }
    if (coin.mintAuthority) {
      reasons.push('Mint authority enabled');
    }
    if (coin.freezeAuthority) {
      reasons.push('Freeze authority enabled');
    }
    if (coin.holderCount > 0) {
      const topHolder = coin.topHolders[0];
      if (topHolder && topHolder.percentage > 10) {
        reasons.push(`Whale holder (${topHolder.percentage.toFixed(1)}%)`);
      }
    }

    // Calculate potential return based on market cap
    const potentialReturn = this.estimatePotentialReturn(coin);

    // Calculate recommended position size
    const recommendedPosition = this.calculatePositionSize(coin, confidence);

    return {
      coin,
      signal,
      confidence,
      reasons: reasons.length > 0 ? reasons : ['Low risk profile'],
      riskScore,
      potentialReturn,
      recommendedPosition,
      timestamp: Date.now(),
    };
  }

  private estimatePotentialReturn(coin: Memecoin): number {
    // Conservative estimate based on market cap
    if (coin.marketCap < 100000) return 500; // 500% potential for micro caps
    if (coin.marketCap < 500000) return 200;
    if (coin.marketCap < 1000000) return 100;
    if (coin.marketCap < 5000000) return 50;
    if (coin.marketCap < 10000000) return 20;
    return 10; // 10% for larger caps
  }

  private calculatePositionSize(coin: Memecoin, confidence: number): number {
    const baseSize = this.config.maxPositionSize;
    const confidenceMultiplier = confidence / 100;
    
    // Reduce position size for higher risk coins
    let position = baseSize * confidenceMultiplier;
    
    // Cap at max single trade
    position = Math.min(position, this.config.maxSingleTrade);
    
    // Further reduce if liquidity is low
    const maxPositionByLiquidity = coin.liquidity * 0.1; // Max 10% of liquidity
    position = Math.min(position, maxPositionByLiquidity);

    return Math.floor(position);
  }

  shouldEnterTrade(signal: TradeSignal): boolean {
    return signal.signal === 'green' || 
           (signal.signal === 'yellow' && this.config.notifyYellow);
  }

  calculateStopLoss(entryPrice: number): number {
    return entryPrice * (1 - this.config.stopLossPercentage / 100);
  }

  calculateTakeProfit(entryPrice: number): number {
    return entryPrice * (1 + this.config.takeProfitPercentage / 100);
  }
}
