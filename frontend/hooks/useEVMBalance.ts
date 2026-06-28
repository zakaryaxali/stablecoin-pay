import { useAccount, useReadContract } from "wagmi";
import { formatUnits } from "viem";
import { BASE_USDC_ADDRESS, USDC_DECIMALS } from "@/config/wagmi";
import { Balance } from "@/types";

// Standard ERC-20 ABI for balanceOf
const ERC20_ABI = [
  {
    name: "balanceOf",
    type: "function",
    stateMutability: "view",
    inputs: [{ name: "account", type: "address" }],
    outputs: [{ name: "balance", type: "uint256" }],
  },
] as const;

interface UseEVMBalanceReturn {
  balance: Balance | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

export function useEVMBalance(): UseEVMBalanceReturn {
  const { address, isConnected } = useAccount();

  const {
    data: rawBalance,
    isLoading,
    error,
    refetch,
  } = useReadContract({
    address: BASE_USDC_ADDRESS,
    abi: ERC20_ABI,
    functionName: "balanceOf",
    args: address ? [address] : undefined,
    query: {
      enabled: isConnected && !!address,
    },
  });

  let balance: Balance | null = null;

  if (rawBalance !== undefined) {
    const amount = parseFloat(formatUnits(rawBalance, USDC_DECIMALS));
    balance = {
      token: "USD Coin",
      symbol: "USDC",
      amount,
      usdValue: amount, // USDC is pegged to USD
      chain: "base",
    };
  }

  return {
    balance,
    isLoading,
    error: error as Error | null,
    refetch,
  };
}
