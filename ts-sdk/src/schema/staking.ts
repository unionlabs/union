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
