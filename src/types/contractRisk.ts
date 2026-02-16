export type RiskSeverity = 'low' | 'medium' | 'high' | 'critical';
export type VerificationStatus = 'verified' | 'partial' | 'unverified' | 'unknown';

export interface VerificationData {
  codeVerified: boolean;
  authorityAddress?: string | null;
  authorityRenounced: boolean;
  mintable: boolean;
  burnable: boolean;
  liquidityLocked: boolean;
  creatorTokenConcentration: number;
  deploymentTime: string;
  timeLockEnabled: boolean;
  feeTransparency: boolean;
  estimatedSellFeePercent: number;
  blacklistEnabled: boolean;
  whitelistEnabled: boolean;
}

export interface HoneypotIndicator {
  category: string;
  description: string;
  severity: RiskSeverity;
  triggered: boolean;
  weight: number;
}

export interface MarketMicrostructure {
  bidAskSpreadPercent: number;
  liquidityDepthUsd: number;
  volume24hUsd: number;
  volumeConsistencyScore: number;
  washTradingScore: number;
  priceManipulationScore: number;
  marketCapUsd: number;
  spreadWithinThreshold: boolean;
  meetsLiquidityRequirement: boolean;
  marketCapVerified: boolean;
}

export interface RiskFactor {
  category: string;
  description: string;
  severity: RiskSeverity;
  score: number;
}

export interface RiskEvent {
  id: number;
  contractAddress: string;
  eventType: string;
  severity: string;
  description: string;
  timestamp: string;
}

export interface ContractAssessment {
  address: string;
  riskScore: number;
  securityScore: number;
  verificationStatus: VerificationStatus;
  verificationData: VerificationData;
  honeypotIndicators: HoneypotIndicator[];
  marketMetrics: MarketMicrostructure;
  riskFactors: RiskFactor[];
  assessedAt: string;
}

export interface ContractRiskDashboard {
  contractAddress: string;
  riskScore: number;
  trustLevel: 'safe' | 'caution' | 'danger' | 'critical';
  verificationStatus: VerificationStatus;
  alerts: RiskEvent[];
  recommendedActions: string[];
}
