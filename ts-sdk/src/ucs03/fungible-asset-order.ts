import { toHex, type Address, type Hex } from "viem"
import * as Either from "effect/Either"
import { Effect, Schema as S, Option, pipe } from "effect"
import { ViemPublicClient, ViemPublicClientSource } from "../evm/client.js"
import { readErc20Meta } from "../evm/erc20.js"
import { predictQuoteToken as predictEvmQuoteToken } from "../evm/quote-token.js"
import { CosmWasmClientContext, CosmWasmClientSource } from "../cosmos/client.js"
import { AptosPublicClient } from "../aptos/client.js"
import { readCw20TokenInfo } from "../cosmos/cw20.js"
import { readFaTokenInfo } from "../aptos/fa.js"
import { predictQuoteToken as predictCosmosQuoteToken } from "../cosmos/quote-token.js"
import { predictQuoteToken as predictAptosQuoteToken } from "../aptos/quote-token.js"
import { FungibleAssetOrder } from "./instruction.js"
import type { AddressCosmosZkgm, AddressEvmZkgm } from "../schema/address.js"
import { ensureHex } from "../utils/index.js"
import type { TokenRawDenom } from "../schema/token.js"
import { graphqlQuoteTokenUnwrapQuery } from "../graphql/unwrapped-quote-token.js"
import type { UniversalChainId } from "../schema/chain.js"
import type { ChannelId } from "../schema/channel.js"

export type FungibleAssetOrderIntent = {
  sender: Address
  receiver: Address
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}

const guardAgainstZeroAmount = (intent: { baseAmount: bigint; quoteAmount: bigint }) => {
  if (intent.baseAmount <= 0n) {
    return Effect.fail(new Error("baseAmount must be greater than zero"))
  }
  return Effect.succeed(intent)
}

/**
 * Creates a fungible asset order from EVM to EVM
 */
export const createEvmToEvmFungibleAssetOrder = (intent: {
  sender: AddressEvmZkgm
  receiver: AddressEvmZkgm
  baseToken: TokenRawDenom
  baseAmount: bigint
  quoteAmount: bigint
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* ViemPublicClientSource).client
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId
    })

    yield* Effect.log("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.log("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.log("predicting quote token on chain")
      finalQuoteToken = yield* predictEvmQuoteToken(intent.baseToken)
      yield* Effect.log("received quote token onchain", finalQuoteToken)
    }

    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        unwrapping ? BigInt(intent.sourceChannelId) : 0n, // path is source channel when unwrapping, else 0
        finalQuoteToken,
        intent.quoteAmount
      ]
    })
  })

/**
 * Creates a fungible asset order from EVM to Cosmos
 */
export const createEvmToCosmosFungibleAssetOrder = (intent: {
  sender: AddressEvmZkgm
  receiver: AddressCosmosZkgm
  baseToken: TokenRawDenom
  baseAmount: bigint
  quoteAmount: bigint
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    yield* Effect.log("creating client")
    const sourceClient = (yield* ViemPublicClientSource).client
    yield* Effect.log("reading erc20 meta")
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient })
    )

    yield* Effect.log(
      "checking if we should unwrap by querying graphql quote token",
      ensureHex(intent.baseToken)
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId
    })

    yield* Effect.log("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.log("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.log("predicting quote token on chain")
      finalQuoteToken = yield* predictCosmosQuoteToken(intent.baseToken)
      yield* Effect.log("received quote token onchain", finalQuoteToken)
    }

    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        unwrapping ? BigInt(intent.sourceChannelId) : 0n, // path is source channel when unwrapping, else 0
        finalQuoteToken,
        intent.quoteAmount
      ]
    })
  }).pipe(Effect.withLogSpan("create fungible asset order"))

/**
 * Creates a fungible asset order from Cosmos to EVM
 */
export const createCosmosToEvmFungibleAssetOrder = (intent: {
  sender: AddressCosmosZkgm
  receiver: AddressEvmZkgm
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client

    const tokenMeta = yield* pipe(
      readCw20TokenInfo(intent.baseToken),
      Effect.provideService(CosmWasmClientContext, { client: sourceClient }),
      Effect.either,
      Effect.map(
        Either.getOrElse(() => ({
          symbol: intent.baseToken,
          name: intent.baseToken,
          decimals: 6
        }))
      )
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId
    })

    yield* Effect.log("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.log("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.log("predicting quote token on chain")
      finalQuoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))
      yield* Effect.log("received quote token onchain", finalQuoteToken)
    }

    // const quoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))

    console.log("here", intent)
    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        unwrapping ? BigInt(intent.sourceChannelId) : 0n, // path is source channel when unwrapping, else 0
        finalQuoteToken,
        intent.quoteAmount
      ]
    })
  })

/**
 * Creates a fungible asset order from Cosmos to Cosmos
 */
export const createCosmosToCosmosFungibleAssetOrder = (intent: {
  sender: AddressCosmosZkgm
  receiver: AddressCosmosZkgm
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client

    const tokenMeta = yield* pipe(
      readCw20TokenInfo(intent.baseToken),
      Effect.provideService(CosmWasmClientContext, { client: sourceClient }),
      Effect.either,
      Effect.map(
        Either.getOrElse(() => ({
          symbol: intent.baseToken,
          name: intent.baseToken,
          decimals: 6
        }))
      )
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId
    })

    yield* Effect.log("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.log("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.log("predicting quote token on chain")
      finalQuoteToken = yield* predictCosmosQuoteToken(intent.baseToken)
      yield* Effect.log("received quote token onchain", finalQuoteToken)
    }

    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        unwrapping ? BigInt(intent.sourceChannelId) : 0n, // path is source channel when unwrapping, else 0
        finalQuoteToken,
        intent.quoteAmount
      ]
    })
  })

/**
 * Creates a fungible asset order from Aptos to Cosmos
 */
export const createCosmosToAptosFungibleAssetOrder = (intent: {
  sender: AddressCosmosZkgm
  receiver: string
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client
    // HACK: special cased for muno for now
    const tokenMeta =
      intent.baseToken === "muno"
        ? {
            symbol: "muno",
            name: "muno",
            decimals: 0
          }
        : yield* readCw20TokenInfo(intent.baseToken).pipe(
            Effect.provideService(CosmWasmClientContext, { client: sourceClient })
          )

    const quoteToken = yield* predictAptosQuoteToken(toHex(intent.baseToken))

    yield* Effect.log("quote token from aptos is", quoteToken, " for base token ", intent.baseToken)

    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        // @ts-expect-error
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        0n, // channel if unwrapping
        quoteToken,
        intent.quoteAmount
      ]
    })
  })

/**
 * Creates a fungible asset order from Cosmos to Aptos
 */
export const createAptosToCosmosFungibleAssetOrder = (intent: {
  sender: string
  receiver: AddressCosmosZkgm
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function* () {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* AptosPublicClient).client
    const tokenMeta = yield* readFaTokenInfo(intent.baseToken).pipe(
      Effect.provideService(AptosPublicClient, { client: sourceClient })
    )
    const quoteToken = yield* predictCosmosQuoteToken(toHex(intent.baseToken))

    return yield* S.decode(FungibleAssetOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        // @ts-expect-error
        intent.sender,
        intent.receiver,
        ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        0n, // channel if unwrapping
        quoteToken,
        intent.quoteAmount
      ]
    })
  })
