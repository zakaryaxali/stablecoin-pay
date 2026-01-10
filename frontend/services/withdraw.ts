import { API_BASE } from "./config";

export interface BuildWithdrawRequest {
  wallet: string;
  amount: number;
  protocol: "kamino" | "save";
}

export interface BuildWithdrawResponse {
  transaction: string; // base64 encoded unsigned transaction
  blockhash: string;
  last_valid_block_height: number;
  protocol: string;
  amount_lamports: number;
}

/**
 * Build an unsigned withdraw transaction on the backend.
 * The transaction is returned as base64 and needs to be signed by the wallet.
 */
export async function buildWithdrawTransaction(
  request: BuildWithdrawRequest
): Promise<BuildWithdrawResponse> {
  const response = await fetch(`${API_BASE}/withdrawals/build`, {
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
