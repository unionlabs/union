import {
  Effect,
  Either as E,
  flow,
  Match,
  Option as O,
  ParseResult,
  pipe,
  Schema as S,
  String as Str,
} from "effect"

export const Erc20 = S.Struct({
  _tag: S.tag("Erc20"),
  address: S.String,
})
export type Erc20 = typeof Erc20.Type

export const CosmosBank = S.Struct({
  _tag: S.tag("CosmosBank"),
  address: S.String,
})
export type CosmosBank = typeof CosmosBank.Type

export const IbcClassic = S.Struct({
  _tag: S.tag("IbcClassic"),
  address: S.String,
})
export type IbcClassic = typeof IbcClassic.Type

export const Any = S.Union(
  Erc20,
  CosmosBank,
  IbcClassic,
)
export type Any = typeof Any.Type

export const TokenFromString = S.transformOrFail(
  S.NonEmptyString,
  Any,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Match.value(fromA),
        // XXX: revise string matching
        Match.when(flow(Str.match(/^0x[a-fA-F0-9]{40}$/), O.isSome), (address) =>
          Effect.succeed(Erc20.make({ address }))),
        Match.when(flow(Str.match(/^[a-zA-Z0-9/.:_-]+$/), O.isSome), (address) =>
          Effect.succeed(CosmosBank.make({ address }))),
        Match.when(flow(Str.match(/^ibc\/[a-fA-F0-9]{64}$/), O.isSome), (address) =>
          Effect.succeed(IbcClassic.make({ address }))),
        Match.orElse(() =>
          ParseResult.fail(new ParseResult.Type(ast, fromA, "No match"))
        ),
      ),
    encode: (toI) => Effect.succeed(toI.address),
  },
)
export type TokenFromString = typeof TokenFromString.Type
