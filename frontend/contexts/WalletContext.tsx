import { FC, ReactNode, useMemo } from "react";
import { Platform } from "react-native";
import {
  ConnectionProvider,
  WalletProvider,
} from "@solana/wallet-adapter-react";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-phantom";
import { clusterApiUrl } from "@solana/web3.js";

// Only import wallet adapter UI styles on web
if (Platform.OS === "web") {
  require("@solana/wallet-adapter-react-ui/styles.css");
}

interface WalletContextProviderProps {
  children: ReactNode;
}

export const WalletContextProvider: FC<WalletContextProviderProps> = ({
  children,
}) => {
  // Use mainnet for production, devnet for development
  const endpoint = useMemo(() => clusterApiUrl("mainnet-beta"), []);

  // Initialize Phantom wallet adapter
  const wallets = useMemo(() => [new PhantomWalletAdapter()], []);

  // Only render wallet provider on web (mobile would use different approach)
  if (Platform.OS !== "web") {
    return <>{children}</>;
  }

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        {children}
      </WalletProvider>
    </ConnectionProvider>
  );
};
