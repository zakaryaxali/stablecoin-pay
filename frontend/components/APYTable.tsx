import { View, Text, Pressable } from "react-native";
import { ApyRate, formatApy, formatTvl, getPlatformName } from "@/services/apy";

interface APYTableProps {
  rates: ApyRate[];
  bestPlatform?: string;
  onDeposit?: (platform: string) => void;
}

export function APYTable({ rates, bestPlatform, onDeposit }: APYTableProps) {
  if (rates.length === 0) {
    return (
      <View className="bg-white rounded-2xl p-6 mx-4 mt-4">
        <Text className="text-gray-500 text-center">No rates available</Text>
      </View>
    );
  }

  return (
    <View className="bg-white rounded-2xl mx-4 mt-4 overflow-hidden">
      {/* Header */}
      <View className="flex-row bg-gray-50 px-4 py-3 border-b border-gray-100">
        <Text className="flex-1 text-gray-500 text-xs font-semibold uppercase">
          Protocol
        </Text>
        <Text className="w-20 text-gray-500 text-xs font-semibold uppercase text-right">
          APY
        </Text>
        <Text className="w-20 text-gray-500 text-xs font-semibold uppercase text-right">
          TVL
        </Text>
        {onDeposit && (
          <Text className="w-20 text-gray-500 text-xs font-semibold uppercase text-right">
            Action
          </Text>
        )}
      </View>

      {/* Rows */}
      {rates.map((rate, index) => {
        const isBest = rate.platform === bestPlatform;
        const isLast = index === rates.length - 1;

        return (
          <View
            key={rate.platform}
            className={`flex-row items-center px-4 py-4 ${
              !isLast ? "border-b border-gray-100" : ""
            } ${isBest ? "bg-green-50" : ""}`}
          >
            {/* Protocol name */}
            <View className="flex-1">
              <View className="flex-row items-center">
                <Text className="text-gray-900 font-semibold">
                  {getPlatformName(rate.platform)}
                </Text>
                {isBest && (
                  <View className="bg-green-500 rounded-full px-2 py-0.5 ml-2">
                    <Text className="text-white text-xs font-medium">Best</Text>
                  </View>
                )}
              </View>
              <Text className="text-gray-400 text-xs mt-0.5">
                {rate.chain.toUpperCase()} · {rate.token}
              </Text>
            </View>

            {/* APY */}
            <View className="w-20 items-end">
              <Text
                className={`font-bold ${
                  isBest ? "text-green-600" : "text-indigo-600"
                }`}
              >
                {formatApy(rate.apy_total)}
              </Text>
              {rate.apy_reward && parseFloat(rate.apy_reward) > 0 && (
                <Text className="text-gray-400 text-xs">
                  +{formatApy(rate.apy_reward)} rewards
                </Text>
              )}
            </View>

            {/* TVL */}
            <Text className="w-20 text-gray-500 text-right">
              {formatTvl(rate.tvl_usd)}
            </Text>

            {/* Deposit button */}
            {onDeposit && (
              <View className="w-20 items-end">
                <Pressable
                  onPress={() => onDeposit(rate.platform)}
                  className="bg-indigo-600 rounded-lg px-3 py-1.5 active:bg-indigo-700"
                >
                  <Text className="text-white text-xs font-semibold">
                    Deposit
                  </Text>
                </Pressable>
              </View>
            )}
          </View>
        );
      })}
    </View>
  );
}
