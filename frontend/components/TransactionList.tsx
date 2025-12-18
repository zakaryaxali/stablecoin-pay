import { FlatList, View, Text, RefreshControl } from "react-native";
import { Transaction } from "@/types";
import { TransactionItem } from "./TransactionItem";

interface TransactionListProps {
  transactions: Transaction[];
  refreshing?: boolean;
  onRefresh?: () => void;
  showHeader?: boolean;
  limit?: number;
}

export function TransactionList({
  transactions,
  refreshing = false,
  onRefresh,
  showHeader = true,
  limit,
}: TransactionListProps) {
  const displayTransactions = limit
    ? transactions.slice(0, limit)
    : transactions;

  return (
    <View className="flex-1 bg-white rounded-t-2xl mt-4">
      {showHeader && (
        <View className="px-4 py-3 border-b border-gray-100">
          <Text className="text-gray-900 font-semibold text-lg">
            Recent Transactions
          </Text>
        </View>
      )}
      <FlatList
        data={displayTransactions}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => <TransactionItem transaction={item} />}
        refreshControl={
          onRefresh ? (
            <RefreshControl refreshing={refreshing} onRefresh={onRefresh} />
          ) : undefined
        }
        ListEmptyComponent={
          <View className="p-8 items-center">
            <Text className="text-gray-400">No transactions yet</Text>
          </View>
        }
      />
    </View>
  );
}
