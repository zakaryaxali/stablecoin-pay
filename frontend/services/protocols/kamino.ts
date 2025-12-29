import { Connection, PublicKey } from "@solana/web3.js";

// USDC mint on mainnet
const USDC_MINT = new PublicKey(
  "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
);

export interface DepositResult {
  success: boolean;
  signature?: string;
  error?: string;
}

export class KaminoService {
  private connection: Connection;

  constructor(connection: Connection) {
    this.connection = connection;
  }

  async getUsdcBalance(userPublicKey: PublicKey): Promise<number> {
    try {
      const tokenAccounts = await this.connection.getParsedTokenAccountsByOwner(
        userPublicKey,
        { mint: USDC_MINT }
      );

      if (tokenAccounts.value.length === 0) {
        return 0;
      }

      const balance =
        tokenAccounts.value[0].account.data.parsed.info.tokenAmount.uiAmount;
      return balance || 0;
    } catch (error) {
      console.error("Failed to fetch USDC balance:", error);
      return 0;
    }
  }
}

// Singleton instance factory
let kaminoServiceInstance: KaminoService | null = null;

export function getKaminoService(connection: Connection): KaminoService {
  if (!kaminoServiceInstance) {
    kaminoServiceInstance = new KaminoService(connection);
  }
  return kaminoServiceInstance;
}
