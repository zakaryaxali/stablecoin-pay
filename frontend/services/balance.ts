import { API_BASE } from "./config";

export interface BalanceResponse {
  address: string;
  token: string;
  symbol: string;
  amount: string;
  usd_value: string;
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
