import {
  FC,
  ReactNode,
  createContext,
  useContext,
  useState,
  useMemo,
} from "react";
import { Platform } from "react-native";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { clusterApiUrl } from "@solana/web3.js";
import { WagmiProvider } from "wagmi";
import { RainbowKitProvider } from "@rainbow-me/rainbowkit";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { wagmiConfig } from "@/config/wagmi";
import { ChainType } from "@/types";

// Only import wallet adapter UI styles on web
if (Platform.OS === "web") {
  require("@solana/wallet-adapter-react-ui/styles.css");
  require("@rainbow-me/rainbowkit/styles.css");
}

// Create a query client for React Query (required by wagmi)
const queryClient = new QueryClient();

interface MultiChainContextValue {
  activeChain: ChainType;
  setActiveChain: (chain: ChainType) => void;
}

const MultiChainContext = createContext<MultiChainContextValue>({
  activeChain: "solana",
  setActiveChain: () => {},
});

export function useMultiChain() {
  return useContext(MultiChainContext);
}

interface MultiChainWalletProviderProps {
  children: ReactNode;
}

export const MultiChainWalletProvider: FC<MultiChainWalletProviderProps> = ({
  children,
}) => {
  const [activeChain, setActiveChain] = useState<ChainType>("solana");

  // Solana configuration
  const endpoint = useMemo(() => clusterApiUrl("mainnet-beta"), []);
  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  const contextValue = useMemo(
    () => ({ activeChain, setActiveChain }),
    [activeChain]
  );

  // Only render providers on web (mobile would use different approach)
  if (Platform.OS !== "web") {
    return (
      <MultiChainContext.Provider value={contextValue}>
        {children}
      </MultiChainContext.Provider>
    );
  }

  return (
    <QueryClientProvider client={queryClient}>
      <WagmiProvider config={wagmiConfig}>
        <RainbowKitProvider>
          <ConnectionProvider endpoint={endpoint}>
            <WalletProvider wallets={wallets} autoConnect>
              <MultiChainContext.Provider value={contextValue}>
                {children}
              </MultiChainContext.Provider>
            </WalletProvider>
          </ConnectionProvider>
        </RainbowKitProvider>
      </WagmiProvider>
    </QueryClientProvider>
  );
};
