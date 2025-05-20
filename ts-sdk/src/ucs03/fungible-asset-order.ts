import { Effect, Option, pipe, Schema as S } from "effect"
import * as Either from "effect/Either"
import { type Address, type Hex, toHex } from "viem"
import { AptosPublicClient } from "../aptos/client.js"
import { readFaTokenInfo } from "../aptos/fa.js"
import { predictQuoteToken as predictAptosQuoteToken } from "../aptos/quote-token.js"
import { CosmWasmClientContext, CosmWasmClientSource } from "../cosmos/client.js"
import { readCw20TokenInfo } from "../cosmos/cw20.js"
import { predictQuoteToken as predictCosmosQuoteToken } from "../cosmos/quote-token.js"
import { ViemPublicClient, ViemPublicClientSource } from "../evm/client.js"
import { readErc20Meta } from "../evm/erc20.js"
import { predictQuoteToken as predictEvmQuoteToken } from "../evm/quote-token.js"
import { graphqlQuoteTokenUnwrapQuery } from "../graphql/unwrapped-quote-token.js"
import { AddressCosmosZkgm, AddressEvmZkgm } from "../schema/address.js"
import { UniversalChainId } from "../schema/chain.js"
import { ChannelId } from "../schema/channel.js"
import { TokenRawDenom } from "../schema/token.js"
import { ensureHex } from "../utils/index.js"
import { FungibleAssetOrder } from "./instruction.js"

const BaseIntent = S.Struct({
  baseAmount: S.BigIntFromSelf.pipe(
    S.greaterThanBigInt(0n),
  ),
  quoteAmount: S.BigIntFromSelf,
  sourceChainId: UniversalChainId,
  sourceChannelId: ChannelId,
})
type BaseIntent = typeof BaseIntent.Type

const guardAgainstZeroAmount = (intent: { baseAmount: bigint; quoteAmount: bigint }) => {
  if (intent.baseAmount <= 0n) {
    return Effect.fail(new Error("baseAmount must be greater than zero"))
  }
  return Effect.succeed(intent)
}

export const EvmToEvmIntent = pipe(
  S.Struct({
    sender: AddressEvmZkgm,
    receiver: AddressEvmZkgm,
    baseToken: TokenRawDenom,
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
export type EvmToEvmIntent = typeof EvmToEvmIntent.Type
/**
 * Creates a fungible asset order from EVM to EVM
 */
export const createEvmToEvmFungibleAssetOrder = (intent: EvmToEvmIntent) =>
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* ViemPublicClientSource).client
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient }),
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    yield* Effect.logTrace("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.logTrace("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.logTrace("predicting quote token on chain")
      finalQuoteToken = yield* predictEvmQuoteToken(intent.baseToken)
      yield* Effect.logTrace("received quote token onchain", finalQuoteToken)
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
        intent.quoteAmount,
      ],
    })
  })

export const EvmToCosmosIntent = pipe(
  S.Struct({
    sender: AddressEvmZkgm,
    receiver: AddressCosmosZkgm,
    baseToken: TokenRawDenom,
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
export type EvmToCosmosIntent = typeof EvmToCosmosIntent.Type
/**
 * Creates a fungible asset order from EVM to Cosmos
 */
export const createEvmToCosmosFungibleAssetOrder = (intent: EvmToCosmosIntent) =>
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    yield* Effect.logTrace("creating client")
    const sourceClient = (yield* ViemPublicClientSource).client
    yield* Effect.logTrace("reading erc20 meta")
    const tokenMeta = yield* readErc20Meta(intent.baseToken as Address).pipe(
      Effect.provideService(ViemPublicClient, { client: sourceClient }),
    )

    yield* Effect.logTrace(
      "checking if we should unwrap by querying graphql quote token",
      ensureHex(intent.baseToken),
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    yield* Effect.logTrace("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.logTrace("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.logTrace("predicting quote token on chain")
      finalQuoteToken = yield* predictCosmosQuoteToken(intent.baseToken)
      yield* Effect.logTrace("received quote token onchain", finalQuoteToken)
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
        intent.quoteAmount,
      ],
    })
  }).pipe(Effect.withLogSpan("create fungible asset order"))

export const CosmosToEvmIntent = pipe(
  S.Struct({
    sender: AddressCosmosZkgm,
    receiver: AddressEvmZkgm,
    baseToken: S.Union(
      TokenRawDenom,
      S.Literal("uxion"),
    ),
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
export type CosmosToEvmIntent = typeof CosmosToEvmIntent.Type
/**
 * Creates a fungible asset order from Cosmos to EVM
 */
export const createCosmosToEvmFungibleAssetOrder = (intent: CosmosToEvmIntent) =>
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client

    const tokenMeta = yield* pipe(
      readCw20TokenInfo(intent.baseToken),
      Effect.provideService(CosmWasmClientContext, { client: sourceClient }),
      Effect.either,
      Effect.map(
        Either.getOrElse(() => ({
          symbol: intent.baseToken === "uxion" ? "XION" : intent.baseToken,
          name: intent.baseToken === "uxion" ? "xion" : intent.baseToken,
          decimals: intent.baseToken === "uxion" ? 0 : 6,
        })),
      ),
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    yield* Effect.logTrace("graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.logTrace("using the graphql quote token unwrapped", graphqlDenom.value)
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.logTrace("predicting quote token on chain")
      finalQuoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))
      yield* Effect.logTrace("received quote token onchain", finalQuoteToken)
    }

    // const quoteToken = yield* predictEvmQuoteToken(toHex(intent.baseToken))

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
        intent.quoteAmount,
      ],
    })
  })

export const CosmosToCosmosIntent = pipe(
  S.Struct({
    sender: AddressCosmosZkgm,
    receiver: AddressCosmosZkgm,
    baseToken: S.Union(
      TokenRawDenom,
      S.Literal("uxion"),
      S.Literal("ubbn"),
    ),
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
export type CosmosToCosmosIntent = typeof CosmosToCosmosIntent.Type
/**
 * Creates a fungible asset order from Cosmos to Cosmos
 */
export const createCosmosToCosmosFungibleAssetOrder = (intent: CosmosToCosmosIntent) =>
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client

    const tokenMeta = yield* pipe(
      readCw20TokenInfo(intent.baseToken),
      Effect.provideService(CosmWasmClientContext, { client: sourceClient }),
      Effect.either,
      Effect.map(
        Either.getOrElse(() => ({
          symbol: intent.baseToken === "uxion" ? "XION" : intent.baseToken,
          name: intent.baseToken === "uxion" ? "xion" : intent.baseToken,
          decimals: intent.baseToken === "uxion" ? 0 : 6,
        })),
      ),
    )

    const queryVariables = {
      baseToken: ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    }

    yield* Effect.logTrace(
      "[createCosmosToCosmosFungibleAssetOrder] query variables",
      queryVariables,
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery(queryVariables)

    yield* Effect.logTrace("[createCosmosToCosmosFungibleAssetOrder] graphql quote", graphqlDenom)
    let finalQuoteToken: Hex
    const unwrapping = Option.isSome(graphqlDenom)
    if (unwrapping) {
      yield* Effect.logTrace(
        "[createCosmosToCosmosFungibleAssetOrder] using the graphql quote token unwrapped",
        graphqlDenom.value,
      )
      finalQuoteToken = graphqlDenom.value
    } else {
      yield* Effect.logTrace(
        "[createCosmosToCosmosFungibleAssetOrder] predicting quote token on chain",
      )
      finalQuoteToken = yield* predictCosmosQuoteToken(ensureHex(intent.baseToken))
      yield* Effect.logTrace(
        "[createCosmosToCosmosFungibleAssetOrder] received quote token onchain",
        finalQuoteToken,
      )
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
        intent.quoteAmount,
      ],
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
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* CosmWasmClientSource).client
    // HACK: special cased for muno for now
    const tokenMeta = intent.baseToken === "muno"
      ? {
        symbol: "muno",
        name: "muno",
        decimals: 0,
      }
      : yield* readCw20TokenInfo(intent.baseToken).pipe(
        Effect.provideService(CosmWasmClientContext, { client: sourceClient }),
      )

    const quoteToken = yield* predictAptosQuoteToken(toHex(intent.baseToken))

    yield* Effect.logTrace(
      "quote token from aptos is",
      quoteToken,
      " for base token ",
      intent.baseToken,
    )

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
        intent.quoteAmount,
      ],
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
  Effect.gen(function*() {
    yield* guardAgainstZeroAmount(intent)
    const sourceClient = (yield* AptosPublicClient).client
    const tokenMeta = yield* readFaTokenInfo(intent.baseToken).pipe(
      Effect.provideService(AptosPublicClient, { client: sourceClient }),
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
        intent.quoteAmount,
      ],
    })
  })
