import * as S from "effect/Schema"
import { Chain, RpcType } from "./chain.js"
import { EvmWethToken, TokenRawAmount, TokenRawDenom } from "./token.js"
import { ChannelId } from "./channel.js"

const BaseTransferFields = {
  sourceChain: Chain.annotations({
    message: () => "sourceChain cant be empty"
  }),
  baseToken: TokenRawDenom.annotations({
    message: () => "baseToken must be a non-empty string (e.g., token address or symbol)"
  }),
  baseAmount: TokenRawAmount.annotations({
    message: () => "baseAmount must be a valid bigint string (e.g., '1000000')"
  }),
  quoteToken: TokenRawDenom.annotations({
    message: () => "quoteToken must be a non-empty string (e.g., token address or symbol)"
  }),
  quoteAmount: TokenRawAmount.annotations({
    message: () => "quoteAmount must be a valid bigint string (e.g., '1000000')"
  }),
  sourceChannelId: ChannelId.annotations({
    message: () => "sourceChannelId must be a non-negative integer"
  }),
  destinationRpcType: RpcType.annotations({
    message: () => "destinationType must be a valid RPC type ('evm', 'cosmos', or 'aptos')"
  }),
  ucs03address: S.String,
  timeoutHeight: S.BigInt, // XXX: Should probably be BigIntFromSelf
  timeoutTimestamp: S.String
}

const EvmTransferS = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("evm").annotations({
    message: () => "sourceRpcType must be 'evm'"
  }),
  wethQuoteToken: EvmWethToken,
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type EvmTransferS = typeof EvmTransferS.Type

export class EVMTransfer extends S.Class<EVMTransfer>("EVMTransfer")(EvmTransferS) {}

const CosmosTransferS = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("cosmos").annotations({
    message: () => "sourceRpcType must be 'cosmos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type CosmosTransferS = typeof CosmosTransferS.Type

export class CosmosTransfer extends S.Class<CosmosTransfer>("CosmosTransfer")(
  CosmosTransferS
) {}

const AptosTransferS = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("aptos").annotations({
    message: () => "sourceRpcType must be 'aptos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type AptosTransferS = typeof AptosTransferS.Type

export class AptosTransfer extends S.Class<AptosTransfer>("AptosTransfer")(
  AptosTransferS
) {}

export const TransferS = S.Union(EVMTransfer, CosmosTransfer, AptosTransfer).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
})
export type TransferS = typeof TransferS.Type

export const ValidTransferS = S.Struct({
  isValid: S.Literal(true),
  args: TransferS
}).annotations({
  identifier: "ValidTransfer",
  title: "Valid Transfer",
  description: "A valid transfer with complete arguments"
})
export type ValidTransferS = typeof ValidTransferS.Type

export class ValidTransfer extends S.Class<ValidTransfer>("ValidTransfer")(
  ValidTransferS
) {}

// Then create the union of those partial schemas
const PartialTransferUnionS = S.Union(
  S.partial(EvmTransferS),
  S.partial(CosmosTransferS),
  S.partial(AptosTransferS)
)
type PartialTransferUnionS = typeof PartialTransferUnionS.Type

// Finally create the NotValidTransfer schema
export const NotValidTransferS = S.Struct({
  isValid: S.Literal(false),
  args: PartialTransferUnionS
}).annotations({
  identifier: "NotValidTransfer",
  title: "Invalid Transfer",
  description: "An invalid transfer with partial arguments"
})
export type NotValidTransferS = typeof NotValidTransferS.Type

export class NotValidTransfer extends S.Class<NotValidTransfer>("NotValidTransfer")(
  NotValidTransferS
) {}
