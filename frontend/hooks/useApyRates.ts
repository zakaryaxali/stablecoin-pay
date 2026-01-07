import { useState, useEffect, useCallback } from "react";
import { ApyRate, getApyRates } from "@/services/apy";

interface UseApyRatesReturn {
  rates: ApyRate[];
  bestPoolId: string | undefined;
  isLoading: boolean;
  isRefreshing: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useApyRates(): UseApyRatesReturn {
  const [rates, setRates] = useState<ApyRate[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchRates = useCallback(async (showRefresh = false) => {
    if (showRefresh) {
      setIsRefreshing(true);
    }
    setError(null);

    try {
      const data = await getApyRates();
      // Sort by APY descending
      const sorted = data.sort(
        (a, b) => parseFloat(b.apy_total) - parseFloat(a.apy_total)
      );
      setRates(sorted);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch rates");
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  }, []);

  useEffect(() => {
    fetchRates();
  }, [fetchRates]);

  const refresh = useCallback(async () => {
    await fetchRates(true);
  }, [fetchRates]);

  // Best pool is the one with highest APY (first after sorting)
  const bestPoolId = rates.length > 0 ? (rates[0].pool_id ?? rates[0].platform) : undefined;

  return {
    rates,
    bestPoolId,
    isLoading,
    isRefreshing,
    error,
    refresh,
  };
}
