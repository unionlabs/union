/**
 * This module constructs fungible asset orders for given chains.
 *
 * @since 2.0.0
 */

import { Effect, pipe, Schema as S } from "effect"
import * as Either from "effect/Either"
import { constant } from "effect/Function"
import * as O from "effect/Option"
import { type Address, type Hex, toHex } from "viem"
import * as Cosmos from "./Cosmos.js"
import * as Evm from "./Evm.js"
import { graphqlQuoteTokenUnwrapQuery } from "./graphql/unwrapped-quote-token.js"
import { AddressCosmosZkgm, AddressEvmZkgm } from "./schema/address.js"
import { UniversalChainId } from "./schema/chain.js"
import { ChannelId } from "./schema/channel.js"
import { TokenRawDenom } from "./schema/token.js"
import * as Sui from "./Sui.js"
import * as Ucs03 from "./Ucs03.js"
import * as Utils from "./Utils.js"

const BaseIntent = S.Struct({
  baseAmount: S.BigIntFromSelf.pipe(
    S.greaterThanBigInt(0n),
  ),
  quoteAmount: S.BigIntFromSelf,
  sourceChainId: UniversalChainId,
  sourceChannelId: ChannelId,
})
type BaseIntent = typeof BaseIntent.Type

/**
 * @category models
 * @since 2.0.0
 */
export const EvmToEvmIntent = pipe(
  S.Struct({
    sender: AddressEvmZkgm,
    receiver: AddressEvmZkgm,
    baseToken: TokenRawDenom,
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
/**
 * @category models
 * @since 2.0.0
 */
export type EvmToEvmIntent = typeof EvmToEvmIntent.Type
/**
 * Creates a fungible asset order from EVM to EVM.
 *
 * @category utils
 * @since 2.0.0
 */
export const evmToEvm = (intent: EvmToEvmIntent) =>
  Effect.gen(function*() {
    yield* S.validate(EvmToEvmIntent)(intent)

    const sourceClient = yield* Evm.PublicClientSource
    const tokenMeta = yield* pipe(
      Evm.readErc20Meta(intent.baseToken as Address, intent.sourceChainId),
      Effect.provideService(Evm.PublicClient, sourceClient),
    )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: Utils.ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    const finalQuoteToken = yield* O.match(graphqlDenom, {
      onSome: Effect.succeed,
      onNone: () => Evm.predictQuoteToken(intent.baseToken),
    })

    // path is source channel when unwrapping, else 0
    const path = O.match(graphqlDenom, {
      onSome: constant(BigInt(intent.sourceChannelId)),
      onNone: constant(0n),
    })

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        Utils.ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        path,
        finalQuoteToken,
        intent.quoteAmount,
      ],
    })
  })

/**
 * @category models
 * @since 2.0.0
 */
export const EvmToCosmosIntent = pipe(
  S.Struct({
    sender: AddressEvmZkgm,
    receiver: AddressCosmosZkgm,
    baseToken: TokenRawDenom,
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
/**
 * @category models
 * @since 2.0.0
 */
export type EvmToCosmosIntent = typeof EvmToCosmosIntent.Type
/**
 * Creates a fungible asset order from EVM to Cosmos
 *
 * @since 2.0.0
 */
export const evmToCosmos = (intent: EvmToCosmosIntent) =>
  Effect.gen(function*() {
    yield* S.validate(EvmToCosmosIntent)(intent)

    const sourceClient = yield* Evm.PublicClientSource
    const tokenMeta = yield* Evm.readErc20Meta(intent.baseToken as Address, intent.sourceChainId)
      .pipe(
        Effect.provideService(Evm.PublicClient, sourceClient),
      )

    const graphqlDenom = yield* graphqlQuoteTokenUnwrapQuery({
      baseToken: Utils.ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    const finalQuoteToken = yield* O.match(graphqlDenom, {
      onSome: Effect.succeed,
      onNone: () => Cosmos.predictQuoteToken(intent.baseToken),
    })

    // path is source channel when unwrapping, else 0
    const path = O.match(graphqlDenom, {
      onSome: constant(BigInt(intent.sourceChannelId)),
      onNone: constant(0n),
    })

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        Utils.ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        path,
        finalQuoteToken,
        intent.quoteAmount,
      ],
    })
  })

/**
 * @category models
 * @since 2.0.0
 */
export const CosmosToEvmIntent = pipe(
  S.Struct({
    sender: AddressCosmosZkgm,
    receiver: AddressEvmZkgm,
    baseToken: S.Union(
      TokenRawDenom,
      S.Literal("uxion"),
      S.Literal("ubbn"),
    ),
  }),
  S.extend(BaseIntent),
  S.asSchema,
)
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosToEvmIntent = typeof CosmosToEvmIntent.Type
/**
 * Creates a fungible asset order from Cosmos to EVM
 *
 * @since 2.0.0
 */
export const cosmosToEvm = (intent: CosmosToEvmIntent) =>
  Effect.gen(function*() {
    yield* S.validate(CosmosToEvmIntent)(intent)

    const sourceClient = yield* Cosmos.ClientSource

    const tokenMeta = yield* pipe(
      Cosmos.readCw20TokenInfo(intent.baseToken),
      Effect.provideService(Cosmos.Client, sourceClient),
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
      baseToken: Utils.ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    yield* Effect.logTrace("graphqlDenom", JSON.stringify(graphqlDenom, null, 2))

    const finalQuoteToken = yield* O.match(graphqlDenom, {
      onSome: Effect.succeed,
      onNone: () => Evm.predictQuoteToken(Utils.ensureHex(intent.baseToken)),
    })

    yield* Effect.logTrace("final quote token", JSON.stringify(finalQuoteToken, null, 2))

    // path is source channel when unwrapping, else 0
    const path = O.match(graphqlDenom, {
      onSome: constant(BigInt(intent.sourceChannelId)),
      onNone: constant(0n),
    })

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        Utils.ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        path,
        finalQuoteToken,
        intent.quoteAmount,
      ],
    })
  })

/**
 * @category models
 * @since 2.0.0
 */
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
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosToCosmosIntent = typeof CosmosToCosmosIntent.Type
/**
 * Creates a fungible asset order from Cosmos to Cosmos
 *
 * @since 2.0.0
 */
export const cosmosToCosmos = (intent: CosmosToCosmosIntent) =>
  Effect.gen(function*() {
    yield* S.validate(CosmosToCosmosIntent)(intent)

    const sourceClient = yield* Cosmos.ClientSource

    const tokenMeta = yield* pipe(
      Cosmos.readCw20TokenInfo(intent.baseToken),
      Effect.provideService(Cosmos.Client, sourceClient),
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
      baseToken: Utils.ensureHex(intent.baseToken),
      sourceChainId: intent.sourceChainId,
      sourceChannelId: intent.sourceChannelId,
    })

    const finalQuoteToken = yield* O.match(graphqlDenom, {
      onSome: Effect.succeed,
      onNone: () => Cosmos.predictQuoteToken(Utils.ensureHex(intent.baseToken)),
    })

    // path is source channel when unwrapping, else 0
    const path = O.match(graphqlDenom, {
      onSome: constant(BigInt(intent.sourceChannelId)),
      onNone: constant(0n),
    })

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver,
        Utils.ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        path,
        finalQuoteToken,
        intent.quoteAmount,
      ],
    })
  })

/**
 * Creates a fungible asset order from Sui to Cosmos
 *
 * @since 2.0.0
 */
export const suiToCosmos = (intent: {
  sender: string
  receiver: AddressCosmosZkgm
  baseTokenType: string // It is the Sui coin type, not a raw denom
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function*() {
    const sourceClient = yield* Sui.PublicClient
    const tokenMeta = yield* pipe(
      Sui.readCoinMetadata(intent.baseTokenType),
      Effect.provideService(Sui.PublicClient, sourceClient),
    )

    if (!tokenMeta) {
      return Effect.fail(new Error(`Token metadata not found for ${intent.baseTokenType}`))
    }
    const baseToken = intent.baseTokenType.split("::")[0]
    const quoteToken = yield* Cosmos.predictQuoteToken(baseToken)

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        // @ts-expect-error
        intent.sender,
        intent.receiver,
        Utils.ensureHex(baseToken),
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
 * Creates a fungible asset order from  Cosmos to Sui
 *
 * @since 2.0.0
 */
export const cosmosToSui = (intent: {
  sender: AddressCosmosZkgm
  receiver: string
  baseToken: string
  baseAmount: bigint
  quoteAmount: bigint
}) =>
  Effect.gen(function*() {
    const sourceClient = yield* Cosmos.ClientSource

    // HACK: special cased for muno for now
    const tokenMeta = intent.baseToken === "muno"
      ? {
        symbol: "muno",
        name: "muno",
        decimals: 6,
      }
      : yield* Cosmos.readCw20TokenInfo(intent.baseToken).pipe(
        Effect.provideService(Cosmos.Client, sourceClient),
      )

    const quoteToken = yield* Sui.predictQuoteToken(toHex(intent.baseToken))

    yield* Effect.logTrace(
      "quote token from sui is",
      quoteToken,
      " for base token ",
      intent.baseToken,
    )

    return yield* S.decode(Ucs03.TokenOrder)({
      _tag: "FungibleAssetOrder",
      operand: [
        intent.sender,
        intent.receiver as Address,
        Utils.ensureHex(intent.baseToken),
        intent.baseAmount,
        tokenMeta.symbol,
        tokenMeta.name,
        tokenMeta.decimals,
        0n, // channel if unwrapping
        quoteToken as Hex,
        intent.quoteAmount,
      ],
    })
  })
