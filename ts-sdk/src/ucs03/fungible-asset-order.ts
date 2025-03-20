import { toHex, type Address, type Hex } from "viem"
import { Effect } from "effect"
import { ViemPublicClient, ViemPublicClientSource } from "../evm/client.js"
import { readErc20Meta } from "../evm/erc20.js"
import { predictQuoteToken as predictEvmQuoteToken } from "../evm/quote-token.js"
import { CosmWasmClientContext, CosmWasmClientSource } from "../cosmos/client.js"
import { readCw20TokenInfo } from "../cosmos/cw20.js"
import { predictQuoteToken as predictCosmosQuoteToken } from "../cosmos/quote-token.js"
import { FungibleAssetOrder } from "./instruction.js"

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
export const createEvmToEvmFungibleAssetOrder = (intent: {
  sender: Address
  receiver: Address
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    const sourceClient = (yield* ViemPublicClientSource).client
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )
    const quoteToken = yield* predictEvmQuoteToken(intent.baseToken)

    return FungibleAssetOrder([
      intent.sender,
      intent.receiver,
      intent.baseToken,
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0n, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ])
  })

/**
 * Creates a fungible asset order from EVM to Cosmos
 */
export const createEvmToCosmosFungibleAssetOrder = (intent: {
  sender: Address
  receiver: string
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    yield* Effect.log("creating client")
    const sourceClient = (yield* ViemPublicClientSource).client
    yield* Effect.log("reading erc20 meta")
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )
    yield* Effect.log("predicting quote token")
    const quoteToken = yield* predictCosmosQuoteToken(intent.baseToken)
    yield* Effect.log("quote token", quoteToken)

    return FungibleAssetOrder([
      intent.sender,
      toHex(intent.receiver),
      intent.baseToken,
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0n, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ])
  }).pipe(Effect.withLogSpan("create fungible asset order"))

/**
 * Creates a fungible asset order from Cosmos to EVM
 */
export const createCosmosToEvmFungibleAssetOrder = (intent: {
  sender: string
  receiver: Address
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    const sourceClient = (yield* CosmWasmClientSource).client
    const tokenMeta = yield* readCw20TokenInfo(intent.baseToken).pipe(
      Effect.provideService(CosmWasmClientContext, { client: sourceClient })
    )
    const quoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))

    return FungibleAssetOrder([
      toHex(intent.sender),
      intent.receiver,
      toHex(intent.baseToken),
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0n, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ])
  })

/**
 * Creates a fungible asset order from Cosmos to Cosmos
 */
export const createCosmosToCosmosFungibleAssetOrder = (intent: {
  sender: string
  receiver: string
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    const sourceClient = (yield* CosmWasmClientSource).client
    const tokenMeta = yield* readCw20TokenInfo(intent.baseToken).pipe(
      Effect.provideService(CosmWasmClientContext, { client: sourceClient })
    )
    const quoteToken = yield* predictCosmosQuoteToken(toHex(intent.baseToken))

    return FungibleAssetOrder([
      toHex(intent.sender),
      toHex(intent.receiver),
      toHex(intent.baseToken),
      intent.baseAmount,
      tokenMeta.symbol,
      tokenMeta.name,
      tokenMeta.decimals,
      0n, // channel if unwrapping
      quoteToken,
      intent.quoteAmount
    ])
  })
