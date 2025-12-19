import { View, ScrollView, Text, ActivityIndicator, RefreshControl } from "react-native";
import { BalanceCard } from "@/components/BalanceCard";
import { TransactionList } from "@/components/TransactionList";
import { useWallet } from "@/hooks/useWallet";
import { mockBalance, mockTransactions } from "@/data/mockData";

// TODO: Add wallet input screen to let user enter their address
const WALLET_ADDRESS = "";

export default function HomeScreen() {
  const { balance, transactions, isLoading, isRefreshing, error, refresh } = useWallet(
    WALLET_ADDRESS
  );

  // Use mock data if no wallet configured or API fails
  const displayBalance = balance || mockBalance;
  const displayTransactions = transactions.length > 0 ? transactions : mockTransactions;
  const usingMockData = !balance || transactions.length === 0;

  if (isLoading) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center">
        <ActivityIndicator size="large" color="#4f46e5" />
        <Text className="text-gray-500 mt-4">Loading wallet data...</Text>
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
        {error && (
          <View className="bg-red-50 px-4 py-3 mx-4 mt-4 rounded-lg">
            <Text className="text-red-600 text-sm">{error}</Text>
          </View>
        )}

        {usingMockData && (
          <View className="bg-yellow-50 px-4 py-3 mx-4 mt-4 rounded-lg">
            <Text className="text-yellow-700 text-sm">
              Using demo data. Update WALLET_ADDRESS in index.tsx to see real data.
            </Text>
          </View>
        )}

        <BalanceCard balance={displayBalance} />
        <TransactionList
          transactions={displayTransactions}
          limit={5}
          showHeader={true}
        />
      </ScrollView>
    </View>
  );
}
