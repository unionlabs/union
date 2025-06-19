/**
 * Wallet verification constants
 */

// The exact message users must sign to verify wallet ownership
export const WALLET_VERIFICATION_MESSAGE =
  "I confirm that I wish to receive any rewards allocated to me in this wallet"

// Supported EVM chain IDs
export const SUPPORTED_CHAINS = {
  "1": "Ethereum Mainnet",
  "11155111": "Sepolia Testnet",
} as const

export type SupportedChainId = keyof typeof SUPPORTED_CHAINS
