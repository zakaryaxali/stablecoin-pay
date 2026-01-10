import { useState, useEffect } from "react";
import {
  View,
  Text,
  ScrollView,
  ActivityIndicator,
  Platform,
  Pressable,
} from "react-native";
import { useLocalSearchParams, useRouter } from "expo-router";
import { useWallet } from "@solana/wallet-adapter-react";
import { WithdrawForm } from "@/components/WithdrawForm";
import { ConnectWallet } from "@/components/ConnectWallet";
import { getPlatformName } from "@/services/apy";
import { useWithdraw } from "@/hooks/useWithdraw";
import { useStakedBalance } from "@/hooks/useStakedBalance";

export default function WithdrawScreen() {
  const { protocol } = useLocalSearchParams<{ protocol: string }>();
  const router = useRouter();

  // Only use wallet hooks on web
  const isWeb = Platform.OS === "web";

  if (!isWeb) {
    return (
      <View className="flex-1 bg-gray-50 items-center justify-center p-4">
        <Text className="text-gray-500 text-center">
          Withdrawals are only available on web. Please use the web version.
        </Text>
      </View>
    );
  }

  return <WithdrawScreenWeb protocol={protocol || ""} />;
}

function WithdrawScreenWeb({ protocol }: { protocol: string }) {
  const router = useRouter();
  const { publicKey, connected } = useWallet();

  // Use extracted hooks for business logic
  const walletAddress = connected && publicKey ? publicKey.toBase58() : null;
  const {
    stakedBalance,
    isLoading: isLoadingBalance,
    error: balanceError,
    refetch: refetchBalance,
  } = useStakedBalance(walletAddress);
  const {
    execute: executeWithdraw,
    isWithdrawing,
    error: withdrawError,
    success,
  } = useWithdraw(protocol);

  // Combine errors from different sources
  const error = balanceError || withdrawError;

  const handleWithdraw = async (amount: number) => {
    await executeWithdraw(amount);
    // Refresh balance after withdraw attempt
    if (walletAddress) {
      refetchBalance();
    }
  };

  const handleCancel = () => {
    router.back();
  };

  const platformName = getPlatformName(protocol);
  const stakedAmount = stakedBalance ? parseFloat(stakedBalance.amount) : 0;
  const stakedSymbol = stakedBalance?.symbol || "kToken";

  // Check if user has staked position
  const hasStakedPosition = stakedAmount > 0;

  return (
    <ScrollView className="flex-1 bg-gray-50">
      {/* Header */}
      <View className="bg-indigo-600 p-6">
        <Text className="text-white text-2xl font-bold">{platformName}</Text>
        <Text className="text-indigo-200 mt-1">
          Withdraw your staked position back to USDC
        </Text>
      </View>

      {/* Wallet Connection */}
      {!connected && (
        <View className="mt-4">
          <ConnectWallet />
        </View>
      )}

      {/* Success Message */}
      {success && (
        <View className="bg-green-50 mx-4 mt-4 p-4 rounded-xl">
          <Text className="text-green-700 font-medium">{success}</Text>
        </View>
      )}

      {/* Error Message */}
      {error && !success && (
        <View className="bg-red-50 mx-4 mt-4 p-4 rounded-xl">
          <Text className="text-red-600">{error}</Text>
        </View>
      )}

      {/* Withdraw Form */}
      {connected && !success && (
        <View className="mt-4">
          {isLoadingBalance ? (
            <View className="items-center py-8">
              <ActivityIndicator color="#4f46e5" />
              <Text className="text-gray-500 mt-2">Loading staked balance...</Text>
            </View>
          ) : !hasStakedPosition ? (
            <View className="bg-yellow-50 mx-4 p-4 rounded-xl">
              <Text className="text-yellow-700 font-medium">No staked position</Text>
              <Text className="text-yellow-600 text-sm mt-1">
                You don't have any {stakedSymbol} to withdraw.
              </Text>
            </View>
          ) : (
            <WithdrawForm
              platform={protocol}
              platformName={platformName}
              stakedBalance={stakedAmount}
              stakedSymbol={stakedSymbol}
              isLoading={isWithdrawing}
              onWithdraw={handleWithdraw}
              onCancel={handleCancel}
            />
          )}
        </View>
      )}

      {/* Back button after success or no position */}
      {(success || (connected && !hasStakedPosition && !isLoadingBalance)) && (
        <Pressable
          className="mx-4 mt-4 bg-indigo-600 rounded-xl py-4 active:bg-indigo-700"
          onPress={handleCancel}
        >
          <Text className="text-white text-center font-semibold">
            Back to Yields
          </Text>
        </Pressable>
      )}
    </ScrollView>
  );
}
