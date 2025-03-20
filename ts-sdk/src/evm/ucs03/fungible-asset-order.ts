import { toHex, type Address, type Hex } from "viem"
import { Context, Effect } from "effect"
import { ViemPublicClient, ViemPublicClientSource } from "../client.js"
import { readErc20Meta } from "../erc20.js"
import { predictQuoteToken as predictEvmQuoteToken } from "../quote-token.js"
import { CosmWasmClientContext, CosmWasmClientSource } from "../../cosmos/client.js"
import { readCw20TokenInfo } from "../../cosmos/cw20.js"
import { predictQuoteToken as predictCosmosQuoteToken } from "../../cosmos/quote-token.js"

export type FungibleAssetOrderIntent = {
  sender: Address
  receiver: Address
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}

/**
 * Creates a fungible asset order from EVM to EVM
 */
export const createEvmToEvmFungibleAssetOrder = (intent: FungibleAssetOrderIntent) =>
  Effect.gen(function* () {
    const sourceClient = (yield* ViemPublicClientSource).client
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )
    const quoteToken = yield* predictEvmQuoteToken(intent.baseToken as Hex)

    return [
      intent.sender,
      intent.receiver,
      intent.baseToken,
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ]
  })

/**
 * Creates a fungible asset order from EVM to Cosmos
 */
export const createEvmToCosmosFungibleAssetOrder = (intent: FungibleAssetOrderIntent) =>
  Effect.gen(function* () {
    const sourceClient = (yield* ViemPublicClientSource).client
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )
    const quoteToken = yield* predictCosmosQuoteToken(intent.baseToken)

    return [
      intent.sender,
      intent.receiver,
      intent.baseToken,
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ]
  })

/**
 * Creates a fungible asset order from Cosmos to EVM
 */
export const createCosmosToEvmFungibleAssetOrder = (intent: FungibleAssetOrderIntent) =>
  Effect.gen(function* () {
    const sourceClient = (yield* CosmWasmClientSource).client
    const tokenMeta = yield* readCw20TokenInfo(intent.baseToken).pipe(
      Effect.provideService(CosmWasmClientContext, { client: sourceClient })
    )
    const quoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))

    return [
      intent.sender,
      intent.receiver,
      toHex(intent.baseToken),
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ]
  })

/**
 * Creates a fungible asset order from Cosmos to Cosmos
 */
export const createCosmosToCosmosFungibleAssetOrder = (intent: FungibleAssetOrderIntent) =>
  Effect.gen(function* () {
    const sourceClient = (yield* CosmWasmClientSource).client
    const tokenMeta = yield* readCw20TokenInfo(intent.baseToken).pipe(
      Effect.provideService(CosmWasmClientContext, { client: sourceClient })
    )
    const quoteToken = yield* predictCosmosQuoteToken(intent.baseToken)

    return [
      intent.sender,
      intent.receiver,
      intent.baseToken,
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ]
  })
