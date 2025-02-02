import type { getChannelInfo } from "@unionlabs/client"
import type { Chain } from "$lib/types.ts"

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
  channel: NonNullable<ReturnType<typeof getChannelInfo>>
  sourceChain: Chain
  destinationChain: Chain
}
