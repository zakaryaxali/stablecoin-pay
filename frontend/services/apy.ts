import { Platform } from "react-native";

// API base URL - matches api.ts pattern
const getBaseUrl = () => {
  if (__DEV__) {
    if (Platform.OS === "android") {
      return "http://10.0.2.2:3000";
    }
    return "http://localhost:3000";
  }
  return "https://api.your-domain.com";
};

const API_BASE = getBaseUrl();

// Types
export interface ApyRate {
  platform: string;
  chain: string;
  token: string;
  apy_total: string;
  apy_base: string | null;
  apy_reward: string | null;
  tvl_usd: string | null;
  fetched_at: string;
}

interface ApyRatesResponse {
  rates: ApyRate[];
  count: number;
}

interface BestApyResponse {
  rate: ApyRate | null;
}

interface ApiError {
  error: string;
}

// Get all current APY rates
export async function getApyRates(): Promise<ApyRate[]> {
  const response = await fetch(`${API_BASE}/apy/rates`);

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || "Failed to fetch APY rates");
  }

  const data: ApyRatesResponse = await response.json();
  return data.rates;
}

// Get best APY rate
export async function getBestApyRate(): Promise<ApyRate | null> {
  const response = await fetch(`${API_BASE}/apy/rates/best`);

  if (!response.ok) {
    const error: ApiError = await response.json();
    throw new Error(error.error || "Failed to fetch best APY rate");
  }

  const data: BestApyResponse = await response.json();
  return data.rate;
}

// Format APY for display (e.g., "3.24" -> "3.24%")
export function formatApy(apy: string): string {
  const value = parseFloat(apy);
  return `${value.toFixed(2)}%`;
}

// Format TVL for display (e.g., "126078976.00" -> "$126.1M")
export function formatTvl(tvl: string | null): string {
  if (!tvl) return "N/A";
  const value = parseFloat(tvl);
  if (value >= 1_000_000_000) {
    return `$${(value / 1_000_000_000).toFixed(1)}B`;
  }
  if (value >= 1_000_000) {
    return `$${(value / 1_000_000).toFixed(1)}M`;
  }
  if (value >= 1_000) {
    return `$${(value / 1_000).toFixed(1)}K`;
  }
  return `$${value.toFixed(0)}`;
}

// Get platform display name
export function getPlatformName(platform: string): string {
  const names: Record<string, string> = {
    kamino: "Kamino",
    save: "Save (Solend)",
    marginfi: "MarginFi",
  };
  return names[platform] || platform;
}
