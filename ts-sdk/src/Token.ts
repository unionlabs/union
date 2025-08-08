/**
 * This module sources {@link Token} data.
 *
 * @since 2.0.0
 */
import { Effect, flow, Match, ParseResult, pipe, Schema as S, Struct } from "effect"
import { constFalse, constTrue } from "effect/Function"

/**
 * @category schemas
 * @since 2.0.0
 */
export const Erc20 = S.Struct({
  _tag: S.tag("Erc20"),
  address: S.String.pipe(
    S.pattern(/^0x[0-9a-fA-F]{40}$/),
    S.annotations({
      examples: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
    }),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type Erc20 = typeof Erc20.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const EvmGas = S.Struct({
  _tag: S.tag("EvmGas"),
  address: S.String.pipe(
    S.pattern(/^0x[eE]{40}$/),
    S.annotations({
      examples: ["0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
    }),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type EvmGas = typeof EvmGas.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const CosmosIbcClassic = S.Struct({
  _tag: S.tag("CosmosIbcClassic"),
  address: S.String.pipe(
    S.pattern(/^ibc\/[0-9A-Fa-f]{64}$/),
    S.annotations({
      examples: [""],
    }),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosIbcClassic = typeof CosmosIbcClassic.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const CosmosTokenFactory = S.Struct({
  _tag: S.tag("CosmosTokenFactory"),
  address: S.String.pipe(
    S.pattern(/^factory\/.+$/),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosTokenFactory = typeof CosmosTokenFactory.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const Cw20 = S.Struct({
  _tag: S.tag("Cw20"),
  address: S.String.pipe(
    S.pattern(/^[a-z0-9]{1,15}1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{38,64}$/),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type Cw20 = typeof Cw20.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const CosmosBank = S.Struct({
  _tag: S.tag("CosmosBank"),
  address: S.String.pipe(
    S.pattern(/^[a-z][a-z0-9]{1,127}$/),
  ),
})
/**
 * @category models
 * @since 2.0.0
 */
export type CosmosBank = typeof CosmosBank.Type

/**
 * @category schemas
 * @since 2.0.0
 */
export const Any = S.Union(
  Erc20,
  EvmGas,
  Cw20,
  CosmosTokenFactory,
  CosmosBank,
  CosmosIbcClassic,
)
/**
 * @category models
 * @since 2.0.0
 */
export type Any = typeof Any.Type

/**
 * @category transformations
 * @since 2.0.0
 */
export const TokenFromString = S.transformOrFail(
  S.String,
  Any,
  {
    decode: (address) =>
      pipe(
        Effect.raceAll([
          S.decodeEither(EvmGas)({ _tag: "EvmGas", address }),
          S.decodeEither(CosmosIbcClassic)({ _tag: "CosmosIbcClassic", address }),
          S.decodeEither(CosmosTokenFactory)({ _tag: "CosmosTokenFactory", address }),
          S.decodeEither(Cw20)({ _tag: "Cw20", address }),
        ]),
        Effect.orElse(() => S.decodeEither(Erc20)({ _tag: "Erc20", address })),
        Effect.orElse(() => S.decodeEither(CosmosBank)({ _tag: "CosmosBank", address })),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: flow(Struct.get("address"), Effect.succeed),
  },
)

/**
 * @category predicates
 * @since 2.0.0
 */
export const isNative = Match.type<Any>().pipe(
  Match.tagsExhaustive({
    CosmosBank: constTrue,
    CosmosIbcClassic: constTrue,
    CosmosTokenFactory: constTrue,
    Cw20: constFalse,
    Erc20: constFalse,
    EvmGas: constTrue,
  }),
)
