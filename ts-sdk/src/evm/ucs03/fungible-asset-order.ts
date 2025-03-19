import type { Address, Hex } from "viem"
import type { FungibleAssetOrder } from "./index.js"
import { Context, Effect } from "effect"
import { ViemPublicClient, ViemPublicClientDestination, ViemPublicClientSource } from "../client.js"
import { readErc20Meta } from "../erc20.js"
import { predictQuoteToken } from "../quote-token.js"
import { DestinationConfig } from "../quote-token.js"

export type FungibleAssetOrderIntent = {
  sender: Address
  receiver: Address
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}

export const createFungibleAssetOrder = (intent: FungibleAssetOrderIntent) =>
  Effect.gen(function* () {
    const sourceClient = (yield* ViemPublicClientSource).client
    const erc20Meta = yield* readErc20Meta(intent.baseToken).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )
    const quoteToken = yield* predictQuoteToken(intent.baseToken)

    return [
      intent.sender,
      intent.receiver,
      intent.baseToken,
      intent.baseAmount,
      erc20Meta.symbol,
      erc20Meta.name,
      erc20Meta.decimals,
      0, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ]
  })
