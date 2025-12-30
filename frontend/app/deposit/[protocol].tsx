import { useState, useEffect } from "react";
import {
  View,
  Text,
  ScrollView,
  ActivityIndicator,
  Platform,
  Pressable,
} from "react-native";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useWallet } from "@solana/wallet-adapter-react";
import { DepositForm } from "@/components/DepositForm";
import { ConnectWallet } from "@/components/ConnectWallet";
import { getPlatformName, formatApy, getApyRates, ApyRate } from "@/services/apy";
import { useDeposit } from "@/hooks/useDeposit";
import { useUsdcBalance } from "@/hooks/useUsdcBalance";

export default function DepositScreen() {
  const { protocol } = useLocalSearchParams<{ protocol: string }>();
  const router = useRouter();

  const [rate, setRate] = useState<ApyRate | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Only use wallet hooks on web
  const isWeb = Platform.OS === "web";

  // Fetch APY rate for this protocol
  useEffect(() => {
    async function fetchRate() {
      try {
        const rates = await getApyRates();
        const protocolRate = rates.find((r) => r.platform === protocol);
        setRate(protocolRate || null);
      } catch (err) {
        setError("Failed to fetch rate");
      } finally {
        setIsLoading(false);
      }
    }
    fetchRate();
  }, [protocol]);

  if (!isWeb) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center p-4">
        <Text className="text-gray-500 text-center">
          Deposits are only available on web. Please use the web version.
        </Text>
      </View>
    );
  }

  return (
    <DepositScreenWeb
      protocol={protocol || ""}
      rate={rate}
      isLoading={isLoading}
      error={error}
    />
  );
}

function DepositScreenWeb({
  protocol,
  rate,
  isLoading: initialLoading,
  error: initialError,
}: {
  protocol: string;
  rate: ApyRate | null;
  isLoading: boolean;
  error: string | null;
}) {
  const router = useRouter();
  const { publicKey, connected } = useWallet();

  // Use extracted hooks for business logic
  const walletAddress = connected && publicKey ? publicKey.toBase58() : null;
  const { balance, isLoading: isLoadingBalance, error: balanceError, refetch: refetchBalance } = useUsdcBalance(walletAddress);
  const { execute: executeDeposit, isDepositing, error: depositError, success } = useDeposit(protocol);

  // Combine errors from different sources
  const error = initialError || balanceError || depositError;

  const handleDeposit = async (amount: number) => {
    await executeDeposit(amount);
    // Refresh balance after deposit attempt
    if (walletAddress) {
      refetchBalance();
    }
  };

  const handleCancel = () => {
    router.back();
  };

  if (initialLoading) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center">
        <ActivityIndicator size="large" color="#4f46e5" />
      </View>
    );
  }

  if (!rate) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center p-4">
        <Text className="text-gray-500 text-center">
          Protocol not found or not supported yet.
        </Text>
      </View>
    );
  }

  const platformName = getPlatformName(protocol);
  const apy = formatApy(rate.apy_total);

  return (
    <ScrollView className="flex-1 bg-gray-50">
      {/* Header */}
      <View className="bg-indigo-600 p-6">
        <Text className="text-white text-2xl font-bold">{platformName}</Text>
        <Text className="text-indigo-200 mt-1">
          Deposit USDC and earn {apy} APY
        </Text>
      </View>

      {/* Wallet Connection */}
      {!connected && (
        <View className="mt-4">
          <ConnectWallet />
        </View>
      )}

      {/* Success Message */}
      {success && (
        <View className="bg-green-50 mx-4 mt-4 p-4 rounded-xl">
          <Text className="text-green-700 font-medium">{success}</Text>
        </View>
      )}

      {/* Error Message */}
      {error && !success && (
        <View className="bg-red-50 mx-4 mt-4 p-4 rounded-xl">
          <Text className="text-red-600">{error}</Text>
        </View>
      )}

      {/* Deposit Form */}
      {connected && !success && (
        <View className="mt-4">
          {isLoadingBalance ? (
            <View className="items-center py-8">
              <ActivityIndicator color="#4f46e5" />
              <Text className="text-gray-500 mt-2">Loading balance...</Text>
            </View>
          ) : (
            <DepositForm
              platform={protocol}
              platformName={platformName}
              apy={apy}
              balance={balance}
              isLoading={isDepositing}
              onDeposit={handleDeposit}
              onCancel={handleCancel}
            />
          )}
        </View>
      )}

      {/* Back button after success */}
      {success && (
        <Pressable
          className="mx-4 mt-4 bg-indigo-600 rounded-xl py-4 active:bg-indigo-700"
          onPress={handleCancel}
        >
          <Text className="text-white text-center font-semibold">
            Back to Yields
          </Text>
        </Pressable>
      )}
    </ScrollView>
  );
}
