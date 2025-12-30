/**
 * Utility functions for deposit amount validation and formatting.
 * USDC uses 6 decimal places.
 */

const USDC_DECIMALS = 6;

/**
 * Sanitizes and formats amount input string.
 * - Removes non-numeric characters except decimal point
 * - Limits to one decimal point
 * - Limits decimal places to USDC precision (6)
 *
 * @returns Formatted string or null if invalid
 */
export function formatAmountInput(text: string): string | null {
  // Only allow numbers and one decimal point
  const cleaned = text.replace(/[^0-9.]/g, "");
  const parts = cleaned.split(".");

  // Multiple decimal points
  if (parts.length > 2) {
    return null;
  }

  // Too many decimal places
  if (parts[1]?.length > USDC_DECIMALS) {
    return null;
  }

  return cleaned;
}

/**
 * Validates that amount is a positive number within balance.
 *
 * @returns Error message or null if valid
 */
export function validateDepositAmount(amount: string, balance: number): string | null {
  const numAmount = parseFloat(amount);

  if (isNaN(numAmount) || numAmount <= 0) {
    return "Please enter a valid amount";
  }

  if (numAmount > balance) {
    return "Insufficient balance";
  }

  return null;
}

/**
 * Checks if amount is valid for deposit.
 */
export function isValidDepositAmount(amount: string, balance: number): boolean {
  const numAmount = parseFloat(amount);
  return !isNaN(numAmount) && numAmount > 0 && numAmount <= balance;
}

/**
 * Formats balance for display with appropriate decimal places.
 */
export function formatBalance(balance: number): string {
  return balance.toLocaleString("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: USDC_DECIMALS,
  });
}
