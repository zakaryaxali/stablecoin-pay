import { API_BASE } from "./config";

export interface BalanceResponse {
  address: string;
  token: string;
  symbol: string;
  amount: string;
  usd_value: string;
}

export interface StakedPosition {
  protocol: string;
  token: string;
  symbol: string;
  amount: string;
  mint: string;
}

export interface StakedBalanceResponse {
  address: string;
  positions: StakedPosition[];
}

/**
 * Fetch USDC balance from backend API.
 * Backend proxies the request to Solana RPC with proper API key.
 */
export async function getUsdcBalance(address: string): Promise<number> {
  try {
    const response = await fetch(`${API_BASE}/wallets/${address}/balance`);

    if (!response.ok) {
      const error = await response.json().catch(() => ({}));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    const data: BalanceResponse = await response.json();
    return parseFloat(data.amount) || 0;
  } catch (error) {
    console.error("Failed to fetch USDC balance:", error);
    throw error;
  }
}

/**
 * Fetch staked (kToken) balance from backend API.
 */
export async function getStakedBalance(address: string): Promise<StakedBalanceResponse> {
  const response = await fetch(`${API_BASE}/wallets/${address}/staked-balance`);

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || `HTTP ${response.status}`);
  }

  return response.json();
}
