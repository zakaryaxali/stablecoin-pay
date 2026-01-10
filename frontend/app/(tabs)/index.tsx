import { View, ScrollView, Text, ActivityIndicator, RefreshControl, Platform } from "react-native";
import { useWallet as useWalletAdapter } from "@solana/wallet-adapter-react";
import { BalanceCard } from "@/components/BalanceCard";
import { TransactionList } from "@/components/TransactionList";
import { ConnectWallet } from "@/components/ConnectWallet";
import { useWallet } from "@/hooks/useWallet";

export default function HomeScreen() {
  // Get connected wallet from adapter (web only)
  const { publicKey, connected } = Platform.OS === "web"
    ? useWalletAdapter()
    : { publicKey: null, connected: false };

  const walletAddress = publicKey?.toBase58() || null;
  const isWalletConnected = Platform.OS === "web" && connected && walletAddress;

  const { balance, transactions, isLoading, isRefreshing, error, refresh } = useWallet(
    walletAddress
  );

  // Show loading only when wallet is connected and data is loading
  if (isWalletConnected && isLoading) {
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
        {/* Wallet connection UI */}
        <ConnectWallet />

        {error && isWalletConnected && (
          <View className="bg-red-50 px-4 py-3 mx-4 mt-4 rounded-lg">
            <Text className="text-red-600 text-sm">{error}</Text>
          </View>
        )}

        {isWalletConnected && balance && (
          <>
            <BalanceCard balance={balance} />
            <TransactionList
              transactions={transactions}
              limit={5}
              showHeader={true}
            />
          </>
        )}

        {!isWalletConnected && (
          <View className="px-4 py-8 items-center">
            <Text className="text-gray-400 text-center">
              Connect your wallet to view your balance
            </Text>
          </View>
        )}
      </ScrollView>
    </View>
  );
}
