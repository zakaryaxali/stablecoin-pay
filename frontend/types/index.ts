export interface Balance {
  token: string;
  symbol: string;
  amount: number;
  usdValue: number;
  chain: "solana";
}

export type TransactionType = "send" | "receive";
export type TransactionStatus = "confirmed" | "pending" | "failed";

export interface Transaction {
  id: string;
  type: TransactionType;
  amount: number;
  token: string;
  symbol: string;
  timestamp: Date;
  status: TransactionStatus;
  counterparty: string;
  signature: string;
}
