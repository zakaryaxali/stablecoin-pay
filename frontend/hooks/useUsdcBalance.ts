import { useState, useEffect, useCallback } from "react";
import { getUsdcBalance } from "@/services/balance";

interface UseUsdcBalanceReturn {
  balance: number;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

export function useUsdcBalance(walletAddress: string | null): UseUsdcBalanceReturn {
  const [balance, setBalance] = useState(0);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchBalance = useCallback(async () => {
    if (!walletAddress) {
      setBalance(0);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const bal = await getUsdcBalance(walletAddress);
      setBalance(bal);
    } catch (err) {
      console.error("Failed to fetch balance:", err);
      setError("Failed to fetch balance. Make sure the backend is running.");
    } finally {
      setIsLoading(false);
    }
  }, [walletAddress]);

  useEffect(() => {
    fetchBalance();
  }, [fetchBalance]);

  return {
    balance,
    isLoading,
    error,
    refetch: fetchBalance,
  };
}
