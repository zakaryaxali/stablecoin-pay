import {
  View,
  ScrollView,
  Text,
  ActivityIndicator,
  RefreshControl,
} from "react-native";
import { useRouter } from "expo-router";
import { APYTable } from "@/components/APYTable";
import { ConnectWallet } from "@/components/ConnectWallet";
import { useApyRates } from "@/hooks/useApyRates";

export default function YieldsScreen() {
  const router = useRouter();
  const { rates, bestPlatform, isLoading, isRefreshing, error, refresh } = useApyRates();

  const handleDeposit = (platform: string) => {
    router.push(`/deposit/${platform}`);
  };

  if (isLoading) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center">
        <ActivityIndicator size="large" color="#4f46e5" />
        <Text className="text-gray-500 mt-4">Loading yield rates...</Text>
      </View>
    );
  }

  return (
    <View className="flex-1 bg-gray-50">
      <ScrollView
        className="flex-1"
        refreshControl={
          <RefreshControl
            refreshing={isRefreshing}
            onRefresh={refresh}
            tintColor="#4f46e5"
          />
        }
      >
        {/* Header */}
        <View className="bg-indigo-600 rounded-2xl p-6 mx-4 mt-4">
          <Text className="text-indigo-200 text-sm font-medium mb-1">
            USDC Lending Rates
          </Text>
          <Text className="text-white text-2xl font-bold">
            Compare & Earn Yield
          </Text>
          <Text className="text-indigo-200 text-sm mt-2">
            Deposit USDC to Solana DeFi protocols and earn passive income
          </Text>
        </View>

        {/* Wallet Connection */}
        <ConnectWallet />

        {/* Error message */}
        {error && (
          <View className="bg-red-50 px-4 py-3 mx-4 mt-4 rounded-lg">
            <Text className="text-red-600 text-sm">{error}</Text>
          </View>
        )}

        {/* APY Table */}
        <APYTable
          rates={rates}
          bestPlatform={bestPlatform}
          onDeposit={handleDeposit}
        />

        {/* Info footer */}
        <View className="px-4 py-6">
          <Text className="text-gray-400 text-xs text-center">
            Rates updated every 5 minutes from DeFiLlama
          </Text>
          <Text className="text-gray-400 text-xs text-center mt-1">
            APY is variable and may change based on market conditions
          </Text>
        </View>
      </ScrollView>
    </View>
  );
}
