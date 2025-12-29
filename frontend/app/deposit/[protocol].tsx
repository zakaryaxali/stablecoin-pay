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
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { DepositForm } from "@/components/DepositForm";
import { ConnectWallet } from "@/components/ConnectWallet";
import { getKaminoService } from "@/services/protocols/kamino";
import { getPlatformName, formatApy, getApyRates, ApyRate } from "@/services/apy";

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
  const { connection } = useConnection();
  const { publicKey, connected } = useWallet();

  const [balance, setBalance] = useState(0);
  const [isLoadingBalance, setIsLoadingBalance] = useState(false);
  const [isDepositing, setIsDepositing] = useState(false);
  const [error, setError] = useState<string | null>(initialError);
  const [success, setSuccess] = useState<string | null>(null);

  // Fetch USDC balance when connected
  useEffect(() => {
    async function fetchBalance() {
      if (!connected || !publicKey) return;

      setIsLoadingBalance(true);
      try {
        const kaminoService = getKaminoService(connection);
        const bal = await kaminoService.getUsdcBalance(publicKey);
        setBalance(bal);
      } catch (err) {
        console.error("Failed to fetch balance:", err);
      } finally {
        setIsLoadingBalance(false);
      }
    }
    fetchBalance();
  }, [connected, publicKey, connection]);

  const handleDeposit = async (amount: number) => {
    if (!publicKey || !connected) {
      setError("Please connect your wallet first");
      return;
    }

    setIsDepositing(true);
    setError(null);

    // Simulate deposit for demo (actual implementation requires backend)
    try {
      await new Promise((resolve) => setTimeout(resolve, 2000));
      setSuccess(
        `Demo: Would deposit ${amount} USDC to ${getPlatformName(protocol)}. Full integration requires backend transaction building.`
      );
    } catch (err) {
      setError("Deposit failed");
    } finally {
      setIsDepositing(false);
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
