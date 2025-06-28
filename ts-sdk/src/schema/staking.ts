import {
  Duration,
  Effect,
  flow,
  Number as N,
  Option as O,
  ParseResult,
  pipe,
  String as Str,
} from "effect"
import * as S from "effect/Schema"
import { Hex } from "./hex.js"
import { Uint256 } from "./uint256.js"

export enum ZkgmStakeState {
  // The position doesn't exist yet.
  UNDEFINED,
  // The tokens are in-flight to be staked.
  STAKING,
  // The tokens are bonded and the position is being rewarded.
  STAKED,
  // The tokens are being unbonded, the position no longer earns rewards.
  UNSTAKING,
  // The tokens has been unstaked and withdrawn.
  UNSTAKED,
}

export const ZkgmStakeStateEnum = S.Enums(ZkgmStakeState)
export type ZkgmStakeStateEnum = typeof ZkgmStakeStateEnum.Type

export const Stake = S.Struct({
  tokenId: Uint256,
  governanceToken: Hex,
  sender: Hex,
  beneficiary: Hex,
  validator: Hex,
  amount: Uint256,
})
export type Stake = typeof Stake.Type

export const Unstake = S.Struct({
  tokenId: Uint256,
  governanceToken: Hex,
  sender: Hex,
  validator: Hex,
  amount: Uint256,
})
export type Unstake = typeof Unstake.Type

const DurationFromProtobufDurationString = S.transformOrFail(
  S.TemplateLiteral(S.String, "s"),
  S.DurationFromSelf,
  {
    decode: (fromA, _, ast) =>
      pipe(
        Str.takeLeft(fromA, fromA.length - 1),
        N.parse,
        O.map(Duration.seconds),
        Effect.catchTags({
          NoSuchElementException: () =>
            Effect.fail(new ParseResult.Type(ast, fromA, "could not parse as number")),
        }),
      ),
    encode: flow(
      Duration.toSeconds,
      (n) => `${n}s` as const,
      Effect.succeed,
    ),
    strict: true,
  },
)

export const Params = S.Struct({
  unbonding_time: DurationFromProtobufDurationString,
  max_validators: S.Int.pipe(
    S.positive(),
  ),
  max_entries: S.Int.pipe(
    S.positive(),
  ),
  historical_entries: S.Int.pipe(
    S.positive(),
  ),
  bond_denom: S.String,
  min_commission_rate: S.BigDecimal,
  jailed_validator_threshold: S.Int.pipe(
    S.positive(),
  ),
  epoch_length: S.NumberFromString.pipe(
    S.int(),
    S.positive(),
  ),
})
export type Params = typeof Params.Type
