import { View, Text } from "react-native";
import { FontAwesome } from "@expo/vector-icons";
import { Transaction } from "@/types";

interface TransactionItemProps {
  transaction: Transaction;
}

function formatDate(date: Date): string {
  const now = new Date();
  const diffDays = Math.floor(
    (now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24)
  );

  if (diffDays === 0) {
    return date.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
    });
  } else if (diffDays === 1) {
    return "Yesterday";
  } else if (diffDays < 7) {
    return date.toLocaleDateString("en-US", { weekday: "short" });
  }
  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

function getStatusColor(status: Transaction["status"]): string {
  switch (status) {
    case "confirmed":
      return "text-green-500";
    case "pending":
      return "text-yellow-500";
    case "failed":
      return "text-red-500";
  }
}

export function TransactionItem({ transaction }: TransactionItemProps) {
  const isReceive = transaction.type === "receive";

  return (
    <View className="flex-row items-center px-4 py-3 bg-white border-b border-gray-100">
      <View
        className={`w-10 h-10 rounded-full items-center justify-center ${
          isReceive ? "bg-green-100" : "bg-red-100"
        }`}
      >
        <FontAwesome
          name={isReceive ? "arrow-down" : "arrow-up"}
          size={16}
          color={isReceive ? "#22c55e" : "#ef4444"}
        />
      </View>

      <View className="flex-1 ml-3">
        <Text className="text-gray-900 font-medium">
          {isReceive ? "Received" : "Sent"} {transaction.symbol}
        </Text>
        <Text className="text-gray-500 text-sm">
          {isReceive ? "From" : "To"} {transaction.counterparty}
        </Text>
      </View>

      <View className="items-end">
        <Text
          className={`font-semibold ${isReceive ? "text-green-600" : "text-gray-900"}`}
        >
          {isReceive ? "+" : "-"}$
          {transaction.amount.toLocaleString("en-US", {
            minimumFractionDigits: 2,
          })}
        </Text>
        <View className="flex-row items-center">
          <Text className={`text-xs ${getStatusColor(transaction.status)}`}>
            {transaction.status}
          </Text>
          <Text className="text-gray-400 text-xs ml-2">
            {formatDate(transaction.timestamp)}
          </Text>
        </View>
      </View>
    </View>
  );
}
