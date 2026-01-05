import { API_BASE } from "./config";

export interface SendTransactionRequest {
  transaction: string; // base64 encoded signed transaction
}

export interface SendTransactionResponse {
  signature: string;
}

export interface ConfirmTransactionRequest {
  signature: string;
  blockhash: string;
  last_valid_block_height: number;
}

export interface ConfirmTransactionResponse {
  confirmed: boolean;
  error?: string;
}

/**
 * Send a signed transaction to Solana via the backend RPC proxy.
 * This avoids browser CORS/rate-limit issues with public RPC endpoints.
 */
export async function sendTransaction(
  transaction: string
): Promise<SendTransactionResponse> {
  const response = await fetch(`${API_BASE}/rpc/send-transaction`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ transaction }),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || `HTTP ${response.status}`);
  }

  return response.json();
}

/**
 * Confirm a transaction on Solana via the backend RPC proxy.
 */
export async function confirmTransaction(
  request: ConfirmTransactionRequest
): Promise<ConfirmTransactionResponse> {
  const response = await fetch(`${API_BASE}/rpc/confirm-transaction`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({}));
    throw new Error(error.message || `HTTP ${response.status}`);
  }

  return response.json();
}
