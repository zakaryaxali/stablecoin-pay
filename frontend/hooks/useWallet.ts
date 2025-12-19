import { useState, useEffect, useCallback } from "react";
import { Balance, Transaction } from "@/types";
import { getBalance, getTransactions, registerWallet } from "@/services/api";

interface UseWalletReturn {
  balance: Balance | null;
  transactions: Transaction[];
  isLoading: boolean;
  isRefreshing: boolean;
  error: string | null;
  refresh: () => Promise<void>;
}

export function useWallet(address: string | null): UseWalletReturn {
  const [balance, setBalance] = useState<Balance | null>(null);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(async (isRefresh = false) => {
    if (!address) {
      setIsLoading(false);
      return;
    }

    if (isRefresh) {
      setIsRefreshing(true);
    } else {
      setIsLoading(true);
    }
    setError(null);

    try {
      // First, ensure wallet is registered
      await registerWallet(address);

      // Fetch balance and transactions in parallel
      const [balanceData, transactionsData] = await Promise.all([
        getBalance(address),
        getTransactions(address, 50),
      ]);

      setBalance(balanceData);
      setTransactions(transactionsData);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to fetch wallet data";
      setError(message);
      console.error("Wallet fetch error:", err);
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  }, [address]);

  // Initial fetch
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  // Refresh function for pull-to-refresh
  const refresh = useCallback(async () => {
    await fetchData(true);
  }, [fetchData]);

  return {
    balance,
    transactions,
    isLoading,
    isRefreshing,
    error,
    refresh,
  };
}
