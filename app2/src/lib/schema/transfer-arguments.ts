import { Schema } from "effect"
import { RpcType } from "$lib/schema/chain"
import { EVMWethToken, TokenRawAmount, TokenRawDenom } from "$lib/schema/token"
import { ChannelId } from "$lib/schema/channel"
import { isValidCanonicalForChain } from "$lib/utils/convert-display";

const BaseTransferFields = {
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
  })
}

const EVMTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "evm", { message: () => "type must be 'evm'" })
  ),
  wethToken: EVMWethToken,
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
}).pipe(
  Schema.filter(data =>
    isValidCanonicalForChain(data.receiver, data.destinationRpcType)
      ? true
      : `receiver must be a valid display address for ${data.destinationRpcType}`
  )
)

export class EVMTransfer extends Schema.Class<EVMTransfer>("EVMTransfer")(EVMTransferSchema) {}

const CosmosTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "cosmos", { message: () => "type must be 'cosmos'" })
  ),
  wethToken: Schema.Null,
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
}).pipe(
  Schema.filter(data =>
    isValidCanonicalForChain(data.receiver, data.destinationRpcType)
      ? true
      : `receiver must be a valid display address for ${data.destinationRpcType}`
  )
)

export class CosmosTransfer extends Schema.Class<CosmosTransfer>("CosmosTransfer")(
  CosmosTransferSchema
) {}

const AptosTransferSchema = Schema.Struct({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "aptos", { message: () => "type must be 'aptos'" })
  ),
  wethToken: Schema.Null,
  receiver: Schema.String.pipe(
    Schema.nonEmptyString({ message: () => "receiver must be a non-empty string" })
  )
}).pipe(
  Schema.filter(data =>
    isValidCanonicalForChain(data.receiver, data.destinationRpcType)
      ? true
      : `receiver must be a valid display address for ${data.destinationRpcType}`
  )
)

export class AptosTransfer extends Schema.Class<AptosTransfer>("AptosTransfer")(
  AptosTransferSchema
) {}

export const TransferSchema = Schema.Union(EVMTransfer, CosmosTransfer, AptosTransfer).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
})

export type Transfer = Schema.Schema.Type<typeof TransferSchema>
export type EVMTransferType = Schema.Schema.Type<typeof EVMTransfer>
export type CosmosTransferType = Schema.Schema.Type<typeof CosmosTransfer>
export type AptosTransferType = Schema.Schema.Type<typeof AptosTransfer>
