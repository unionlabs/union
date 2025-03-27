import { Schema } from "effect"
import { Chain, RpcType } from "$lib/schema/chain"
import { EvmWethToken, TokenRawAmount, TokenRawDenom } from "$lib/schema/token"
import { ChannelId } from "$lib/schema/channel"

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
  ucs03address: Schema.String,
  timeoutHeight: Schema.BigInt,
  timeoutTimestamp: Schema.String
}

const EvmTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: Schema.Literal("evm").annotations({
    message: () => "sourceRpcType must be 'evm'"
  }),
  wethQuoteToken: EvmWethToken,
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})

export class EVMTransfer extends Schema.Class<EVMTransfer>("EVMTransfer")(EvmTransferSchema) {}

const CosmosTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: Schema.Literal("cosmos").annotations({
    message: () => "sourceRpcType must be 'cosmos'"
  }),
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})

export class CosmosTransfer extends Schema.Class<CosmosTransfer>("CosmosTransfer")(
  CosmosTransferSchema
) {}

const AptosTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: Schema.Literal("aptos").annotations({
    message: () => "sourceRpcType must be 'aptos'"
  }),
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
})

export class AptosTransfer extends Schema.Class<AptosTransfer>("AptosTransfer")(
  AptosTransferSchema
) {}

export const TransferSchema = Schema.Union(EVMTransfer, CosmosTransfer, AptosTransfer).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
})

export const ValidTransferSchema = Schema.Struct({
  isValid: Schema.Literal(true),
  args: TransferSchema
}).annotations({
  identifier: "ValidTransfer",
  title: "Valid Transfer",
  description: "A valid transfer with complete arguments"
})

export class ValidTransfer extends Schema.Class<ValidTransfer>("ValidTransfer")(
  ValidTransferSchema
) {}

// Then create the union of those partial schemas
const PartialTransferUnionSchema = Schema.Union(
  Schema.partial(EvmTransferSchema),
  Schema.partial(CosmosTransferSchema),
  Schema.partial(AptosTransferSchema)
)

// Finally create the NotValidTransfer schema
export const NotValidTransferSchema = Schema.Struct({
  isValid: Schema.Literal(false),
  args: PartialTransferUnionSchema
}).annotations({
  identifier: "NotValidTransfer",
  title: "Invalid Transfer",
  description: "An invalid transfer with partial arguments"
})

export class NotValidTransfer extends Schema.Class<NotValidTransfer>("NotValidTransfer")(
  NotValidTransferSchema
) {}
