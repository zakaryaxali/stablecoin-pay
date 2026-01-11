import { View, Text } from "react-native";
import { FontAwesome } from "@expo/vector-icons";
import { Transaction, StakingProtocol } from "@/types";

interface TransactionItemProps {
  transaction: Transaction;
}

// Protocol display configuration
const PROTOCOL_CONFIG: Record<StakingProtocol, { name: string; color: string; bgColor: string }> = {
  kamino: { name: "Kamino", color: "#7C3AED", bgColor: "bg-purple-100" },
  save: { name: "Save", color: "#059669", bgColor: "bg-emerald-100" },
};

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

function getTransactionLabel(transaction: Transaction): string {
  const { type, protocol, symbol } = transaction;

  if (protocol) {
    const protocolName = PROTOCOL_CONFIG[protocol].name;
    return type === "receive"
      ? `Withdrew from ${protocolName}`
      : `Deposited to ${protocolName}`;
  }

  return type === "receive" ? `Received ${symbol}` : `Sent ${symbol}`;
}

function getIconConfig(transaction: Transaction): {
  iconName: "arrow-down" | "arrow-up" | "university";
  color: string;
  bgColor: string;
} {
  const { type, protocol } = transaction;

  if (protocol) {
    const config = PROTOCOL_CONFIG[protocol];
    return {
      iconName: "university",
      color: config.color,
      bgColor: config.bgColor
    };
  }

  return type === "receive"
    ? { iconName: "arrow-down", color: "#22c55e", bgColor: "bg-green-100" }
    : { iconName: "arrow-up", color: "#ef4444", bgColor: "bg-red-100" };
}

function shortenAddress(address: string): string {
  if (address.length <= 12) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

export function TransactionItem({ transaction }: TransactionItemProps) {
  const isReceive = transaction.type === "receive";
  const iconConfig = getIconConfig(transaction);
  const label = getTransactionLabel(transaction);

  return (
    <View className="flex-row items-center px-4 py-3 bg-white border-b border-gray-100">
      <View
        className={`w-10 h-10 rounded-full items-center justify-center ${iconConfig.bgColor}`}
      >
        <FontAwesome
          name={iconConfig.iconName}
          size={16}
          color={iconConfig.color}
        />
      </View>

      <View className="flex-1 ml-3">
        <Text className="text-gray-900 font-medium">{label}</Text>
        <Text className="text-gray-500 text-sm">
          {transaction.protocol
            ? shortenAddress(transaction.counterparty)
            : `${isReceive ? "From" : "To"} ${transaction.counterparty}`
          }
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
