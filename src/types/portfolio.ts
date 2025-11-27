export interface CorrelationMatrix {
  symbols: string[];
  matrix: number[][];
  calculatedAt: string;
}

export interface DiversificationMetrics {
  score: number;
  effectiveN: number;
  avgCorrelation: number;
  concentrationRisk: number;
}

export interface RiskConcentration {
  symbol: string;
  allocation: number;
  riskLevel: string;
  recommendation: string;
}

export interface SharpeMetrics {
  sharpeRatio: number;
  annualizedReturn: number;
  annualizedVolatility: number;
  riskFreeRate: number;
}

export interface FactorExposure {
  name: string;
  beta: number;
  exposure: number;
}

export interface FactorAnalysis {
  factors: FactorExposure[];
  marketBeta: number;
  systematicRisk: number;
  specificRisk: number;
}

export interface PortfolioAnalytics {
  correlation: CorrelationMatrix;
  diversification: DiversificationMetrics;
  concentration: RiskConcentration[];
  sharpe: SharpeMetrics;
  factors: FactorAnalysis;
  calculatedAt: string;
}

export interface SectorAllocation {
  sector: string;
  allocation: number;
  percentage: number;
  value: number;
  symbols: string[];
}

export interface ConcentrationAlert {
  id: string;
  symbol: string;
  allocation: number;
  severity: string;
  message: string;
  threshold: number;
  createdAt: string;
}

export interface Position {
  symbol: string;
  amount: number;
  allocation: number;
  value: number;
  pnl?: number;
  pnlPercent?: number;
}
