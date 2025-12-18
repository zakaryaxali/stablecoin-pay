import { View, ScrollView } from "react-native";
import { BalanceCard } from "@/components/BalanceCard";
import { TransactionList } from "@/components/TransactionList";
import { mockBalance, mockTransactions } from "@/data/mockData";

export default function HomeScreen() {
  return (
    <View className="flex-1 bg-gray-50">
      <ScrollView className="flex-1">
        <BalanceCard balance={mockBalance} />
        <TransactionList
          transactions={mockTransactions}
          limit={5}
          showHeader={true}
        />
      </ScrollView>
    </View>
  );
}
