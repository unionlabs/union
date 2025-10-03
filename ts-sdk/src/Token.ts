/**
 * This module sources {@link Token} data.
 *
 * @since 2.0.0
 */
import { isValidSuiAddress, normalizeSuiAddress } from "@mysten/sui/utils"
import { Effect, flow, Hash, Match, ParseResult, pipe, Schema as S, Struct } from "effect"
import { constFalse, constTrue } from "effect/Function"
import * as Chain from "./schema/chain.js"
import * as Hex from "./schema/hex.js"
import * as Utils from "./Utils.js"

/**
 * @category schemas
 * @since 2.0.0
 */
export class SuiCoin extends S.TaggedClass<SuiCoin>()("SuiCoin", {
  address: S.String.pipe(
    S.filter((value) => {
      const parts = value.split("::")
      if (parts.length !== 3) {
        return false
      }
      const [addr, module, name] = parts

      const ident = /^[A-Za-z_][A-Za-z0-9_]*$/
      if (!ident.test(module) || !ident.test(name)) {
        return false
      }

      const norm = normalizeSuiAddress(addr)
      return isValidSuiAddress(norm)
    }, {
      description:
        "Sui coin type in the form 0x<hex>::<Module>::<Name> with a valid Sui address and Move identifiers",
    }),
    S.annotations({
      examples: [
        "0x2::sui::SUI",
        "0x9003c05db750fe8fb33d8e9a7de814b2ca1af024dc67e06f8529260b03d86fdd::usdt_faucet::USDT_FAUCET",
      ],
    }),
  ),
}) {
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class Erc20 extends S.TaggedClass<Erc20>()("Erc20", {
  address: S.String.pipe(
    S.pattern(/^0x[0-9a-fA-F]{40}$/),
    S.annotations({
      examples: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
    }),
    S.filter((_): _ is `0x${string}` => true),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class EvmGas extends S.TaggedClass<EvmGas>()("EvmGas", {
  address: S.String.pipe(
    S.pattern(/^0x[eE]{40}$/),
    S.annotations({
      examples: ["0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"],
    }),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class CosmosIbcClassic extends S.TaggedClass<CosmosIbcClassic>()("CosmosIbcClassic", {
  address: S.String.pipe(
    S.pattern(/^ibc\/[0-9A-Fa-f]{64}$/),
    S.annotations({
      examples: [""],
    }),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class CosmosTokenFactory extends S.TaggedClass<CosmosTokenFactory>()("CosmosTokenFactory", {
  address: S.String.pipe(
    S.pattern(/^factory\/.+$/),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class Cw20 extends S.TaggedClass<Cw20>()("Cw20", {
  address: S.String.pipe(
    S.pattern(/^[a-z0-9]{1,15}1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{38,64}$/),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

/**
 * @category schemas
 * @since 2.0.0
 */
export class CosmosBank extends S.TaggedClass<CosmosBank>()("CosmosBank", {
  // XXX: address incorrect semantically
  address: S.String.pipe(
    S.pattern(/^[a-z][a-z0-9]{1,127}$/),
  ),
}) {
  /**
   * @since 2.0.0
   */
  [Hash.symbol](): number {
    return Hash.string(this.address)
  }
}

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
  SuiCoin,
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
          S.decodeEither(SuiCoin)({ _tag: "SuiCoin", address }),
        ]),
        Effect.orElse(() => S.decodeEither(Erc20)({ _tag: "Erc20", address })),
        Effect.orElse(() => S.decodeEither(CosmosBank)({ _tag: "CosmosBank", address })),
        Effect.catchTag("ParseError", (error) => ParseResult.fail(error.issue)),
      ),
    encode: flow(Struct.get("address"), Effect.succeed),
  },
)

/**
 * @since 2.0.0
 */
export const AnyFromEncoded = (rpcType: Chain.RpcType) =>
  S.transformOrFail(
    Hex.Hex,
    Any,
    {
      decode: (fromA, options, ast, fromI) => {
        return Match.value(rpcType).pipe(
          Match.when("evm", () => S.decode(TokenFromString)(fromA)),
          Match.when("cosmos", () =>
            pipe(
              fromA,
              S.decode(S.compose(
                Hex.StringFromHex,
                TokenFromString,
              )),
            )),
          Match.when("aptos", (fromA) =>
            Effect.fail(new ParseResult.Type(ast, fromA, "Aptos not supported."))),
          Match.when("sui", () =>
            pipe(fromA, S.decode(S.compose(Hex.StringFromHex, TokenFromString)))),
          Match.exhaustive,
          Effect.catchTag("ParseError", (error) =>
            ParseResult.fail(error.issue)),
        )
      },
      encode: (toI, toA) => {
        // TODO: do encode
        return Effect.succeed(Utils.ensureHex(toI.address))
      },
    },
  )

export const normalizeSuiTypeTag = (t: string): string => {
  const [addr, mod, name] = t.split("::")
  return `${normalizeSuiAddress(addr)}::${mod}::${name}`
}

const isNativeSui = (t: string): boolean => {
  // compare on normalized address to avoid short/long mismatch
  const norm = normalizeSuiTypeTag(t)
  return norm === "0x2::sui::SUI"
}

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
    SuiCoin: (t) => isNativeSui(t.address),
  }),
)
