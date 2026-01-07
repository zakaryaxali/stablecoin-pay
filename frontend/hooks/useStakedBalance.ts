import { useState, useEffect, useCallback } from "react";
import { getStakedBalance, StakedBalanceResponse } from "@/services/balance";

interface UseStakedBalanceReturn {
  stakedBalance: StakedBalanceResponse | null;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

export function useStakedBalance(walletAddress: string | null): UseStakedBalanceReturn {
  const [stakedBalance, setStakedBalance] = useState<StakedBalanceResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStakedBalance = useCallback(async () => {
    if (!walletAddress) {
      setStakedBalance(null);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const balance = await getStakedBalance(walletAddress);
      setStakedBalance(balance);
    } catch (err) {
      console.error("Failed to fetch staked balance:", err);
      setError("Failed to fetch staked balance");
      setStakedBalance(null);
    } finally {
      setIsLoading(false);
    }
  }, [walletAddress]);

  useEffect(() => {
    fetchStakedBalance();
  }, [fetchStakedBalance]);

  return {
    stakedBalance,
    isLoading,
    error,
    refetch: fetchStakedBalance,
  };
}
