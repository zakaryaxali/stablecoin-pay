import { useState, useCallback } from "react";
import { View } from "react-native";
import { TransactionList } from "@/components/TransactionList";
import { mockTransactions } from "@/data/mockData";

export default function TransactionsScreen() {
  const [refreshing, setRefreshing] = useState(false);

  const onRefresh = useCallback(() => {
    setRefreshing(true);
    // Simulate API call
    setTimeout(() => {
      setRefreshing(false);
    }, 1000);
  }, []);

  return (
    <View className="flex-1 bg-gray-50">
      <TransactionList
        transactions={mockTransactions}
        refreshing={refreshing}
        onRefresh={onRefresh}
        showHeader={false}
      />
    </View>
  );
}
