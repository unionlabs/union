import { BigDecimal, DateTime, pipe } from "effect"
import * as B from "effect/Boolean"
import * as O from "effect/Option"
import * as S from "effect/Schema"
import * as Utils from "../Utils.js"
import { Chain, UniversalChainId } from "./chain.js"
import { Hex } from "./hex.js"
import { PacketHash } from "./packet.js"
import { Token, TokenRawDenom } from "./token.js"
import { TransactionHash } from "./transaction.js"
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

export class Bond extends S.TaggedClass<Bond>("Bond")("Bond", {
  packet_hash: PacketHash,
  delivery_packet_hash: S.OptionFromNullOr(PacketHash),
  bond_success: S.OptionFromNullOr(S.Boolean.pipe(
    S.annotations({
      description: "the batch doing the bond didn't timeout or negative ack",
    }),
  )),
  delivery_success: S.OptionFromNullOr(S.Boolean.pipe(
    S.annotations({
      description: "the instruction doing the delivery didn't timeout or negative ack",
    }),
  )),
  packet_shape: S.String,
  source_universal_chain_id: UniversalChainId,
  remote_universal_chain_id: UniversalChainId,
  destination_universal_chain_id: UniversalChainId,
  sender_canonical: S.String,
  sender_display: S.String,
  sender_zkgm: S.String,
  receiver_canonical: S.String,
  receiver_display: S.String,
  receiver_zkgm: S.String,
  base_token: TokenRawDenom,
  base_amount: S.BigInt.pipe(
    S.annotations({
      arbitrary: () => (fc) => fc.bigInt({ min: 1n, max: 10n ** 18n - 1n }),
    }),
  ),
  quote_token: TokenRawDenom,
  quote_amount: S.BigInt,
  remote_base_token: S.String,
  remote_base_amount: S.BigInt,
  remote_quote_token: S.String,
  remote_quote_amount: S.BigInt,
  bond_send_timestamp: S.DateTimeUtc,
  bond_send_transaction_hash: TransactionHash,
  bond_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  bond_recv_transaction_hash: S.OptionFromNullOr(TransactionHash),
  bond_timeout_timestamp: S.OptionFromNullOr(S.DateTimeUtc).pipe(
    S.annotations({
      description: "none means no timeout",
    }),
  ),
  bond_timeout_transaction_hash: S.OptionFromNullOr(TransactionHash),
  delivery_send_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  delivery_send_transaction_hash: S.OptionFromNullOr(TransactionHash),
  delivery_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  delivery_recv_transaction_hash: S.OptionFromNullOr(TransactionHash),
  delivery_timeout_timestamp: S.OptionFromNullOr(S.DateTimeUtc).pipe(
    S.annotations({
      description: "none means no timeout",
    }),
  ),
  delivery_timeout_transaction_hash: S.OptionFromNullOr(TransactionHash),
  sort_order: S.String,
  source_chain: Chain,
  destination_chain: Chain,
  // bond_traces: S.Array(PacketTrace),
  // delivery_traces: S.Array(PacketTrace),
  base_token_meta: Token,
  quote_token_meta: S.OptionFromNullOr(Token),
  remote_base_token_meta: Token,
  remote_quote_token_meta: S.OptionFromNullOr(Token),
}) {
  get sortDate() {
    return this.bond_send_timestamp
  }
  get amountFormatted() {
    return pipe(
      this.base_amount,
      BigDecimal.fromBigInt,
      BigDecimal.unsafeDivide(BigDecimal.make(1n, -O.getOrThrow(this.base_token_meta.decimals))),
      BigDecimal.format,
    )
  }
  get sendTimestampFormatted() {
    return pipe(
      this.bond_send_timestamp,
      DateTime.formatIso,
    )
  }
  get status() {
    return pipe(
      this.bond_success,
      O.map(B.match({
        onTrue: () => "success" as const,
        onFalse: () => "failure" as const,
      })),
      O.getOrElse(() => "pending" as const),
    )
  }
}

export class Unbond extends S.TaggedClass<Unbond>("Unbond")("Unbond", {
  packet_hash: PacketHash,
  success: S.OptionFromNullOr(S.Boolean),
  packet_shape: S.String,
  source_universal_chain_id: UniversalChainId,
  destination_universal_chain_id: UniversalChainId,
  sender_canonical: S.String,
  sender_display: S.String,
  sender_zkgm: S.String,
  base_token: TokenRawDenom,
  base_amount: S.BigInt,
  unbond_send_timestamp: S.DateTimeUtc,
  unbond_send_transaction_hash: TransactionHash,
  unbond_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  unbond_recv_transaction_hash: S.OptionFromNullOr(TransactionHash),
  unbond_timeout_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  unbond_timeout_transaction_hash: S.OptionFromNullOr(TransactionHash),
  sort_order: S.String,
  // traces: S.Array(PacketTrace),
  source_chain: Chain,
  destination_chain: Chain,
  base_token_meta: Token,
}) {
  get sortDate() {
    return this.unbond_send_timestamp
  }
  get amountFormatted() {
    return pipe(
      this.base_amount,
      BigDecimal.fromBigInt,
      BigDecimal.unsafeDivide(BigDecimal.make(1n, -O.getOrThrow(this.base_token_meta.decimals))),
      Utils.formatBigDecimal,
    )
  }
  get sendTimestampFormatted() {
    return pipe(
      this.unbond_send_timestamp,
      DateTime.formatIso,
    )
  }
  get status() {
    return pipe(
      this.success,
      O.map(B.match({
        onTrue: () => "success" as const,
        onFalse: () => "failure" as const,
      })),
      O.getOrElse(() => "pending" as const),
    )
  }
}

export class Withdrawal extends S.TaggedClass<Withdrawal>("Withdrawal")("Withdrawal", {
  packet_hash: PacketHash,
  packet_shape: S.String,
  source_universal_chain_id: UniversalChainId,
  destination_universal_chain_id: UniversalChainId,
  staker_canonical: S.String,
  staker_display: S.String,
  staker_zkgm: S.String,
  quote_token: TokenRawDenom,
  quote_amount: S.BigInt,
  withdraw_send_timestamp: S.DateTimeUtc,
  withdraw_send_transaction_hash: TransactionHash,
  withdraw_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  withdraw_recv_transaction_hash: S.OptionFromNullOr(TransactionHash),
  withdraw_timeout_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  withdraw_timeout_transaction_hash: S.OptionFromNullOr(TransactionHash),
  sort_order: S.String,
  source_chain: Chain,
  destination_chain: Chain,
  quote_token_meta: Token,
}) {
  get sortDate() {
    return this.withdraw_send_timestamp
  }
  get amountFormatted() {
    return pipe(
      this.quote_amount,
      BigDecimal.fromBigInt,
      BigDecimal.unsafeDivide(BigDecimal.make(1n, -O.getOrThrow(this.quote_token_meta.decimals))),
      Utils.formatBigDecimal,
    )
  }
  get sendTimestampFormatted() {
    return pipe(
      this.withdraw_send_timestamp,
      DateTime.formatIso,
    )
  }
}
