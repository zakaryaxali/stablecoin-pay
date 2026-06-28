import { View, Text, Pressable } from "react-native";
import { useMultiChain } from "@/contexts/MultiChainWalletContext";
import { ChainType } from "@/types";

interface ChainOption {
  id: ChainType;
  name: string;
  color: string;
  bgColor: string;
}

const CHAINS: ChainOption[] = [
  { id: "solana", name: "Solana", color: "#9945FF", bgColor: "bg-purple-100" },
  { id: "base", name: "Base", color: "#0052FF", bgColor: "bg-blue-100" },
];

export function ChainSelector() {
  const { activeChain, setActiveChain } = useMultiChain();

  return (
    <View className="flex-row bg-gray-100 rounded-lg p-1 mx-4 mt-4">
      {CHAINS.map((chain) => {
        const isActive = activeChain === chain.id;
        return (
          <Pressable
            key={chain.id}
            onPress={() => setActiveChain(chain.id)}
            className={`flex-1 py-2 px-4 rounded-md ${
              isActive ? "bg-white shadow-sm" : ""
            }`}
          >
            <Text
              className={`text-center font-medium ${
                isActive ? "text-gray-900" : "text-gray-500"
              }`}
              style={isActive ? { color: chain.color } : undefined}
            >
              {chain.name}
            </Text>
          </Pressable>
        );
      })}
    </View>
  );
}
