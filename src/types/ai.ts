export interface ChatMessage {
  role: string;
  content: string;
}

export interface ReasoningStep {
  step: number;
  description: string;
  confidence: number;
}

export interface ChatResponse {
  content: string;
  reasoning?: ReasoningStep[];
  metadata?: Record<string, unknown>;
}

export interface OptimizationAction {
  type: string;
  token: string;
  amount: number;
  reason: string;
}

export interface OptimizationRecommendation {
  action: string;
  symbol: string;
  amount?: number;
  reason: string;
}

export interface PortfolioOptimization {
  id: string;
  timestamp: string;
  currentAllocation: Record<string, number>;
  suggestedAllocation: Record<string, number>;
  expectedReturn: number;
  riskScore: number;
  reasoning: string[];
  actions: OptimizationAction[];
  recommendations: OptimizationRecommendation[];
}

export interface PatternWarning {
  id: string;
  timestamp: string;
  pattern: string;
  severity: string;
  tokens: string[];
  description: string;
  recommendation: string;
  confidence: number;
}

export interface StreamingMetadata {
  streamId: string;
  eventName: string;
  isStreaming: boolean;
  currentChunk: string;
}
