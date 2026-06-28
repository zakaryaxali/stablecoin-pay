import { http, createConfig } from "wagmi";
import { base } from "wagmi/chains";
import { metaMask } from "wagmi/connectors";

// Base USDC contract address (native Circle-issued USDC)
export const BASE_USDC_ADDRESS = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913" as const;

// USDC has 6 decimals on Base (same as Solana)
export const USDC_DECIMALS = 6;

export const wagmiConfig = createConfig({
  chains: [base],
  connectors: [metaMask()],
  transports: {
    [base.id]: http("https://mainnet.base.org"),
  },
  ssr: false, // Disable SSR for React Native/Expo web
});

// Re-export chain for convenience
export { base };
