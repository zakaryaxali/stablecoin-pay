import { useState } from "react";
import {
  View,
  Text,
  TextInput,
  Pressable,
  ActivityIndicator,
} from "react-native";
import {
  formatAmountInput,
  validateDepositAmount,
  isValidDepositAmount,
  formatBalance,
} from "@/utils/depositValidation";

interface WithdrawFormProps {
  platform: string;
  platformName: string;
  stakedBalance: number;
  stakedSymbol: string;
  isLoading: boolean;
  onWithdraw: (amount: number) => Promise<void>;
  onCancel: () => void;
}

export function WithdrawForm({
  platform,
  platformName,
  stakedBalance,
  stakedSymbol,
  isLoading,
  onWithdraw,
  onCancel,
}: WithdrawFormProps) {
  const [amount, setAmount] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleAmountChange = (text: string) => {
    const formatted = formatAmountInput(text);
    if (formatted !== null) {
      setAmount(formatted);
      setError(null);
    }
  };

  const handleMaxPress = () => {
    setAmount(stakedBalance.toString());
    setError(null);
  };

  const handleWithdraw = async () => {
    const validationError = validateDepositAmount(amount, stakedBalance);
    if (validationError) {
      setError(validationError);
      return;
    }

    try {
      await onWithdraw(parseFloat(amount));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Withdrawal failed");
    }
  };

  const isValidAmount = isValidDepositAmount(amount, stakedBalance);

  return (
    <View className="bg-white rounded-2xl p-6 mx-4">
      {/* Header */}
      <View className="mb-6">
        <Text className="text-gray-900 text-xl font-bold">
          Withdraw from {platformName}
        </Text>
        <Text className="text-gray-500 text-sm mt-1">
          Redeem your staked tokens back to USDC
        </Text>
      </View>

      {/* Staked Balance */}
      <View className="bg-green-50 rounded-xl p-4 mb-4">
        <Text className="text-green-700 text-xs mb-1">Staked Balance</Text>
        <Text className="text-green-900 text-lg font-semibold">
          {formatBalance(stakedBalance)} {stakedSymbol}
        </Text>
      </View>

      {/* Amount Input */}
      <View className="mb-4">
        <Text className="text-gray-700 text-sm font-medium mb-2">
          Amount to Withdraw
        </Text>
        <View className="flex-row items-center bg-gray-50 rounded-xl border border-gray-200">
          <TextInput
            className="flex-1 px-4 py-3 text-lg text-gray-900"
            placeholder="0.00"
            placeholderTextColor="#9CA3AF"
            keyboardType="decimal-pad"
            value={amount}
            onChangeText={handleAmountChange}
            editable={!isLoading}
          />
          <Pressable
            onPress={handleMaxPress}
            disabled={isLoading}
            className="px-4 py-2 mr-2"
          >
            <Text className="text-indigo-600 font-semibold">MAX</Text>
          </Pressable>
        </View>
      </View>

      {/* Error */}
      {error && (
        <View className="bg-red-50 rounded-lg p-3 mb-4">
          <Text className="text-red-600 text-sm">{error}</Text>
        </View>
      )}

      {/* Buttons */}
      <View className="flex-row gap-3">
        <Pressable
          onPress={onCancel}
          disabled={isLoading}
          className="flex-1 bg-gray-100 rounded-xl py-4 active:bg-gray-200"
        >
          <Text className="text-gray-700 text-center font-semibold">
            Cancel
          </Text>
        </Pressable>
        <Pressable
          onPress={handleWithdraw}
          disabled={!isValidAmount || isLoading}
          className={`flex-1 rounded-xl py-4 ${
            isValidAmount && !isLoading
              ? "bg-indigo-600 active:bg-indigo-700"
              : "bg-gray-300"
          }`}
        >
          {isLoading ? (
            <ActivityIndicator color="white" />
          ) : (
            <Text className="text-white text-center font-semibold">
              Withdraw
            </Text>
          )}
        </Pressable>
      </View>

      {/* Disclaimer */}
      <Text className="text-gray-400 text-xs text-center mt-4">
        You will receive USDC in exchange for your {stakedSymbol} tokens.
        This action cannot be undone.
      </Text>
    </View>
  );
}
