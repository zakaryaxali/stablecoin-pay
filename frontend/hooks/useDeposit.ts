import { useState, useCallback } from "react";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { Transaction } from "@solana/web3.js";
import { buildDepositTransaction } from "@/services/deposit";
import { getPlatformName } from "@/services/apy";

interface UseDepositReturn {
  execute: (amount: number) => Promise<void>;
  isDepositing: boolean;
  error: string | null;
  success: string | null;
  signature: string | null;
  reset: () => void;
}

export function useDeposit(protocol: string): UseDepositReturn {
  const { publicKey, connected, signTransaction } = useWallet();
  const { connection } = useConnection();

  const [isDepositing, setIsDepositing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);

  const reset = useCallback(() => {
    setError(null);
    setSuccess(null);
    setSignature(null);
  }, []);

  const execute = useCallback(async (amount: number) => {
    if (!publicKey || !connected) {
      setError("Please connect your wallet first");
      return;
    }

    if (!signTransaction) {
      setError("Wallet does not support signing transactions");
      return;
    }

    setIsDepositing(true);
    setError(null);
    setSuccess(null);
    setSignature(null);

    try {
      // 1. Build unsigned transaction on backend
      let buildResult;
      try {
        buildResult = await buildDepositTransaction({
          wallet: publicKey.toBase58(),
          amount,
          protocol: protocol as "kamino" | "save",
        });
      } catch (err: any) {
        const message = err.message?.includes("fetch")
          ? "Cannot connect to server. Please check if the backend is running."
          : err.message || "Failed to build transaction";
        throw new Error(message);
      }

      // 2. Deserialize the transaction
      const txBytes = Uint8Array.from(atob(buildResult.transaction), c => c.charCodeAt(0));
      const transaction = Transaction.from(txBytes);

      // 3. Sign the transaction with the wallet
      let signedTransaction;
      try {
        signedTransaction = await signTransaction(transaction);
      } catch (err: any) {
        if (err.message?.includes("User rejected") || err.message?.includes("cancelled")) {
          throw new Error("Transaction cancelled by user");
        }
        throw new Error("Wallet failed to sign: " + (err.message || "Unknown error"));
      }

      // 4. Send the signed transaction to Solana
      let txSignature;
      try {
        txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      } catch (err: any) {
        if (err.message?.includes("insufficient")) {
          throw new Error("Insufficient USDC balance for this deposit");
        }
        if (err.message?.includes("0x1")) {
          throw new Error("Insufficient SOL for transaction fees");
        }
        throw new Error("Transaction failed: " + (err.message || "Unknown error"));
      }

      // 5. Wait for confirmation (using modern API with blockhash context)
      try {
        await connection.confirmTransaction({
          signature: txSignature,
          blockhash: buildResult.blockhash,
          lastValidBlockHeight: buildResult.last_valid_block_height,
        }, "confirmed");
      } catch (err: any) {
        throw new Error(
          `Transaction sent but confirmation timed out. Check explorer: ${txSignature.slice(0, 16)}...`
        );
      }

      setSignature(txSignature);
      setSuccess(
        `Successfully deposited ${amount} USDC to ${getPlatformName(protocol)}! Transaction: ${txSignature.slice(0, 8)}...`
      );
    } catch (err: any) {
      console.error("Deposit failed:", err);
      setError(err.message || "Deposit failed. Please try again.");
    } finally {
      setIsDepositing(false);
    }
  }, [publicKey, connected, signTransaction, connection, protocol]);

  return {
    execute,
    isDepositing,
    error,
    success,
    signature,
    reset,
  };
}
