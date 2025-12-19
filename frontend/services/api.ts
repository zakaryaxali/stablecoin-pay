import { Platform } from "react-native";
import { Balance, Transaction, TransactionType, TransactionStatus } from "@/types";

// API base URL - localhost for iOS simulator, 10.0.2.2 for Android emulator
const getBaseUrl = () => {
  if (__DEV__) {
    // Android emulator uses 10.0.2.2 to reach host machine
    // iOS simulator and web use localhost
    if (Platform.OS === "android") {
      return "http://10.0.2.2:3000";
    }
    return "http://localhost:3000";
  }
  // Production URL - update this when you deploy
  return "https://api.your-domain.com";
};

const API_BASE = getBaseUrl();

// API Response Types (match backend)
interface ApiBalanceResponse {
  address: string;
  token: string;
  symbol: string;
  amount: string;
  usd_value: string;
}

interface ApiTransaction {
  signature: string;
  wallet_address: string;
  tx_type: "send" | "receive";
  amount: string;
  token_mint: string;
  counterparty: string;
  status: "confirmed" | "pending" | "failed";
  block_time: string;
  created_at: string;
}

interface ApiTransactionsResponse {
  transactions: ApiTransaction[];
  count: number;
}

interface ApiError {
  error: string;
}

// Convert API balance response to frontend Balance type
function mapBalanceResponse(response: ApiBalanceResponse): Balance {
  return {
    token: response.token,
    symbol: response.symbol,
    amount: parseFloat(response.amount),
    usdValue: parseFloat(response.usd_value),
    chain: "solana",
  };
}

// Convert API transaction to frontend Transaction type
function mapTransaction(tx: ApiTransaction): Transaction {
  return {
    id: tx.signature,
    type: tx.tx_type as TransactionType,
    amount: parseFloat(tx.amount),
    token: "USD Coin",
    symbol: "USDC",
    timestamp: new Date(tx.block_time),
    status: tx.status as TransactionStatus,
    counterparty: tx.counterparty,
    signature: tx.signature,
  };
}

// Register a wallet to track
export async function registerWallet(address: string): Promise<void> {
  const response = await fetch(`${API_BASE}/wallets`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ address }),
  });

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || "Failed to register wallet");
  }
}

// Get wallet balance
export async function getBalance(address: string): Promise<Balance> {
  const response = await fetch(`${API_BASE}/wallets/${address}/balance`);

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || "Failed to fetch balance");
  }

  const data: ApiBalanceResponse = await response.json();
  return mapBalanceResponse(data);
}

// Get wallet transactions
export async function getTransactions(
  address: string,
  limit?: number,
  offset?: number
): Promise<Transaction[]> {
  const params = new URLSearchParams();
  if (limit) params.append("limit", limit.toString());
  if (offset) params.append("offset", offset.toString());

  const url = `${API_BASE}/wallets/${address}/transactions${params.toString() ? `?${params}` : ""}`;
  const response = await fetch(url);

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || "Failed to fetch transactions");
  }

  const data: ApiTransactionsResponse = await response.json();
  return data.transactions.map(mapTransaction);
}

// Health check
export async function healthCheck(): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE}/health`);
    return response.ok;
  } catch {
    return false;
  }
}
