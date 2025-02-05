import type { getChannelInfo } from "@unionlabs/client"
import type { Chain, UserAddresses } from "$lib/types.ts"
import type { FormFields } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { Result } from "neverthrow"

export type TransferArgs =
  | {
      baseToken: string
      baseAmount: bigint
      quoteToken: string
      quoteAmount: bigint
      receiver: string
      sourceChannelId: number
      ucs03address: string
    }
  | "NO_QUOTE_AVAILABLE"

export type TransferContext = {
  channel: ReturnType<typeof getChannelInfo>
  sourceChain: Chain
  destinationChain: Chain
}

export type BaseToken = {
  denom: string
  balance: string
}

export interface Intents {
  sourceChain: Chain | null
  destinationChain: Chain | null
  baseTokens: Array<{ denom: string; balance: string }>
  baseToken: { denom: string; balance: string } | null
  baseTokenInfo: TokenInfo | null
  channel: NonNullable<ReturnType<typeof getChannelInfo>> | null
  receiver: string | null
  ucs03address: string | null
  amount: string
  ownWallet: string | null
  quoteToken: string | "NO_QUOTE_AVAILABLE" | null
}

export interface TokenInfo {
  combined: {
    decimals: number
  }
}

// Add this interface to represent the balances structure
export interface ChainBalance {
  data?: {
    chain_id: string
    balances: Record<string, string>
  }
}

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidationContext {
  userAddress: UserAddresses
  baseTokenInfo?: TokenInfo | null
  quoteToken: Result<QuoteResponse, Error> | null
}

export type QuoteResponse =
  | {
      quote_token: string
      type: "UNWRAPPED" | "NEW_WRAPPED"
    }
  | {
      type: "NO_QUOTE_AVAILABLE"
    }
