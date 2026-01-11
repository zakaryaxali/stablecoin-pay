import { useCallback } from "react";
import {
  View,
  ScrollView,
  Text,
  ActivityIndicator,
  RefreshControl,
  Pressable,
} from "react-native";
import { useRouter, useFocusEffect } from "expo-router";
import { useWallet } from "@solana/wallet-adapter-react";
import { APYTable } from "@/components/APYTable";
import { ConnectWallet } from "@/components/ConnectWallet";
import { useApyRates } from "@/hooks/useApyRates";
import { useStakedBalance } from "@/hooks/useStakedBalance";

export default function YieldsScreen() {
  const router = useRouter();
  const { publicKey } = useWallet();
  const { rates, bestPoolId, isLoading, isRefreshing, error, refresh: refreshApy } = useApyRates();
  const { stakedBalance, refetch: refetchStaked } = useStakedBalance(publicKey?.toBase58() ?? null);

  const refresh = async () => {
    await Promise.all([refreshApy(), refetchStaked()]);
  };

  // Refetch staked balance when screen comes into focus (e.g., after deposit/withdraw)
  useFocusEffect(
    useCallback(() => {
      refetchStaked();
    }, [refetchStaked])
  );

  const positions = stakedBalance?.positions ?? [];
  const hasStakedPositions = positions.length > 0;

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
        {/* Wallet Connection */}
        <ConnectWallet />

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

        {/* Staked Positions */}
        {hasStakedPositions && (
          <View className="mx-4 mt-4">
            <Text className="text-green-800 text-sm font-medium mb-2">
              Your Staked Positions
            </Text>
            {positions.map((position, index) => (
              <View
                key={position.protocol}
                className={`bg-green-50 border border-green-200 rounded-xl p-4 ${index > 0 ? 'mt-2' : ''}`}
              >
                <View className="flex-row justify-between items-start">
                  <View>
                    <View className="flex-row items-baseline">
                      <Text className="text-green-900 text-2xl font-bold">
                        {parseFloat(position.amount).toFixed(2)}
                      </Text>
                      <Text className="text-green-700 text-sm ml-2">
                        {position.symbol}
                      </Text>
                    </View>
                    <Text className="text-green-600 text-xs mt-1">
                      on {position.protocol.charAt(0).toUpperCase() + position.protocol.slice(1)}
                    </Text>
                  </View>
                  <Pressable
                    className="bg-green-600 px-4 py-2 rounded-lg active:bg-green-700"
                    onPress={() => router.push(`/withdraw/${position.protocol}`)}
                  >
                    <Text className="text-white font-semibold text-sm">Withdraw</Text>
                  </Pressable>
                </View>
              </View>
            ))}
          </View>
        )}

        {/* Error message */}
        {error && (
          <View className="bg-red-50 px-4 py-3 mx-4 mt-4 rounded-lg">
            <Text className="text-red-600 text-sm">{error}</Text>
          </View>
        )}

        {/* APY Table */}
        <APYTable
          rates={rates}
          bestPoolId={bestPoolId}
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
