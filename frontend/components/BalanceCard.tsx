import { View, Text } from "react-native";
import { Balance } from "@/types";

interface BalanceCardProps {
  balance: Balance;
}

export function BalanceCard({ balance }: BalanceCardProps) {
  return (
    <View className="bg-indigo-600 rounded-2xl p-6 mx-4 mt-4">
      <Text className="text-indigo-200 text-sm font-medium mb-1">
        Total Balance
      </Text>
      <Text className="text-white text-4xl font-bold mb-2">
        ${balance.usdValue.toLocaleString("en-US", { minimumFractionDigits: 2 })}
      </Text>
      <View className="flex-row items-center">
        <View className="bg-indigo-500 rounded-full px-3 py-1">
          <Text className="text-white text-sm font-medium">
            {balance.amount.toLocaleString("en-US", { minimumFractionDigits: 2 })}{" "}
            {balance.symbol}
          </Text>
        </View>
        <Text className="text-indigo-200 text-xs ml-2 capitalize">
          on {balance.chain}
        </Text>
      </View>
    </View>
  );
}
