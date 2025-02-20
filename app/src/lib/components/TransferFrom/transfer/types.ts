import type { getChannelInfo } from "@unionlabs/client"
import type { Chain } from "$lib/types.ts"
import type { Balance } from "$lib/stores/balances"
import type { Readable } from "svelte/store"

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

export interface Intents {
  sourceChain: Chain | null
  destinationChain: Chain | null
  baseTokens: Array<{ denom: string; balance: Balance }>
  baseToken: { denom: string; balance: Balance } | null
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

export type QuoteResponse =
  | {
      quote_token: string
      type: "UNWRAPPED" | "NEW_WRAPPED"
    }
  | {
      type: "NO_QUOTE_AVAILABLE"
    }

export type Nullable<T> = T | null
export type DerivedSource = Readable<Nullable<string>>
export type QuoteTokenType = "UNWRAPPED" | "NEW_WRAPPED" | "NO_QUOTE_AVAILABLE"
export type QuoteData =
  | { quote_token: string; type: Extract<QuoteTokenType, "UNWRAPPED" | "NEW_WRAPPED"> }
  | { type: Extract<QuoteTokenType, "NO_QUOTE_AVAILABLE"> }
