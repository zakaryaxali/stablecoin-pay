import { View, Text, Pressable, Platform } from "react-native";
import { useWallet } from "@solana/wallet-adapter-react";

export function ConnectWallet() {
  // Only use wallet hooks on web
  if (Platform.OS !== "web") {
    return (
      <View className="bg-gray-100 rounded-lg px-4 py-3 mx-4 mt-4">
        <Text className="text-gray-500 text-center text-sm">
          Wallet connection is only available on web
        </Text>
      </View>
    );
  }

  return <ConnectWalletWeb />;
}

function ConnectWalletWeb() {
  const { publicKey, connected, connecting, disconnect, select, wallets } =
    useWallet();

  const handleConnect = () => {
    // Select Phantom wallet
    const phantom = wallets.find((w) => w.adapter.name === "Phantom");
    if (phantom) {
      select(phantom.adapter.name);
    }
  };

  const handleDisconnect = () => {
    disconnect();
  };

  // Truncate address for display
  const truncatedAddress = publicKey
    ? `${publicKey.toBase58().slice(0, 4)}...${publicKey.toBase58().slice(-4)}`
    : null;

  if (connected && publicKey) {
    return (
      <View className="bg-green-50 rounded-xl p-4 mx-4 mt-4 flex-row items-center justify-between">
        <View>
          <Text className="text-green-800 text-xs font-medium">Connected</Text>
          <Text className="text-green-600 font-mono text-sm">
            {truncatedAddress}
          </Text>
        </View>
        <Pressable
          onPress={handleDisconnect}
          className="bg-green-600 rounded-lg px-4 py-2 active:bg-green-700"
        >
          <Text className="text-white font-semibold text-sm">Disconnect</Text>
        </Pressable>
      </View>
    );
  }

  return (
    <View className="mx-4 mt-4">
      <Pressable
        onPress={handleConnect}
        disabled={connecting}
        className={`rounded-xl p-4 flex-row items-center justify-center ${
          connecting ? "bg-gray-300" : "bg-purple-600 active:bg-purple-700"
        }`}
      >
        {connecting ? (
          <Text className="text-white font-semibold">Connecting...</Text>
        ) : (
          <>
            <Text className="text-white font-semibold">
              Connect Phantom Wallet
            </Text>
          </>
        )}
      </Pressable>
      <Text className="text-gray-400 text-xs text-center mt-2">
        Connect your wallet to deposit USDC
      </Text>
    </View>
  );
}
