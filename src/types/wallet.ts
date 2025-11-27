export interface TokenBalance {
  mint: string;
  symbol: string;
  name: string;
  balance: number;
  decimals: number;
  uiAmount: number;
  usdValue: number;
  price?: number;
  change24h: number;
  logoUri?: string;
  lastUpdated: string;
}

export interface TransactionFeeEstimate {
  baseFee: number;
  priorityFee: number;
  totalFee: number;
  estimatedUnits: number;
}

export interface SendTransactionInput {
  recipient: string;
  amount: number;
  tokenMint?: string;
  memo?: string;
}

export interface AddressBookContact {
  id: string;
  address: string;
  label: string;
  nickname?: string;
  notes?: string;
  createdAt: string;
  updatedAt: string;
  lastUsed?: string;
  transactionCount: number;
  tags: string[];
}

export interface AddContactRequest {
  address: string;
  label: string;
  nickname?: string;
  notes?: string;
  tags: string[];
}

export interface UpdateContactRequest {
  contactId: string;
  label?: string;
  nickname?: string | null;
  notes?: string | null;
  tags?: string[];
}

export interface QRCodeData {
  address: string;
  amount?: number;
  label?: string;
  message?: string;
  spl?: string;
}

export interface SolanaPayQR {
  url: string;
  qrData: string;
  recipient: string;
  amount?: number;
  splToken?: string;
  reference?: string;
  label?: string;
  message?: string;
  memo?: string;
}

export type SwapStatus = 'pending' | 'completed' | 'failed';

export interface SwapHistoryEntry {
  id: string;
  fromToken: string;
  toToken: string;
  fromAmount: number;
  toAmount: number;
  rate: number;
  fee: number;
  priceImpact: number;
  txSignature?: string;
  timestamp: string;
  status: SwapStatus;
}

export interface BridgeProvider {
  id: string;
  name: string;
  logo: string;
  supportedChains: string[];
  fees: {
    percentage: number;
    fixed: number;
  };
  estimatedTime: string;
}
