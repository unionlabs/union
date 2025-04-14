import * as S from "effect/Schema"
import { Chain, RpcType } from "./chain.js"
import { TokenRawAmount, TokenRawDenom } from "./token.js"
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
}

const EvmTransferSchema = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("evm").annotations({
    message: () => "sourceRpcType must be 'evm'"
  }),
  // wethBaseToken: EvmWethToken,
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type EvmTransferSchema = typeof EvmTransferSchema.Type

export class EVMTransfer extends S.Class<EVMTransfer>("EVMTransfer")(EvmTransferSchema) {}

const CosmosTransferSchema = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("cosmos").annotations({
    message: () => "sourceRpcType must be 'cosmos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type CosmosTransferSchema = typeof CosmosTransferSchema.Type

export class CosmosTransfer extends S.Class<CosmosTransfer>("CosmosTransfer")(
  CosmosTransferSchema
) {}

const AptosTransferSchema = S.Struct({
  ...BaseTransferFields,
  sourceRpcType: S.Literal("aptos").annotations({
    message: () => "sourceRpcType must be 'aptos'"
  }),
  receiver: S.String.pipe(
    S.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})
type AptosTransferSchema = typeof AptosTransferSchema.Type

export class AptosTransfer extends S.Class<AptosTransfer>("AptosTransfer")(AptosTransferSchema) {}

export const TransferSchema = S.Union(EVMTransfer, CosmosTransfer, AptosTransfer).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
})
export type TransferS = typeof TransferSchema.Type

export const ValidTransferS = S.Struct({
  isValid: S.Literal(true),
  args: TransferSchema
}).annotations({
  identifier: "ValidTransfer",
  title: "Valid Transfer",
  description: "A valid transfer with complete arguments"
})
export type ValidTransferS = typeof ValidTransferS.Type

export class ValidTransfer extends S.Class<ValidTransfer>("ValidTransfer")(ValidTransferS) {}

// Then create the union of those partial schemas
const PartialTransferUnionS = S.Union(
  S.partial(EvmTransferSchema),
  S.partial(CosmosTransferSchema),
  S.partial(AptosTransferSchema)
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
