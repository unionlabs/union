import { Schema } from "effect"
import {
  AddressAptosCanonical,
  AddressCosmosCanonical,
  AddressEvmCanonical,
  ReceiverAddress
} from "$lib/schema/address"
import { RpcType } from "$lib/schema/chain"
import { EVMWethToken, TokenRawAmount, TokenRawDenom } from "$lib/schema/token"
import { ChannelId } from "$lib/schema/channel"

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

export class EVMTransfer extends Schema.Class<EVMTransfer>("EVMTransfer")({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "evm"),
    Schema.annotations({ message: () => "type must be 'evm'" })
  ),
  wethToken: EVMWethToken,
  receiver: ReceiverAddress,
  ucs03address: AddressEvmCanonical.pipe(
    Schema.annotations({
      message: () =>
        "ucs03address must be a valid EVM Zkgm address (e.g., 0x followed by 40 hex chars)"
    })
  )
}) {}

export class CosmosTransfer extends Schema.Class<CosmosTransfer>("CosmosTransfer")({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "cosmos"),
    Schema.annotations({ message: () => "type must be 'cosmos'" })
  ),
  wethToken: Schema.Null,
  receiver: ReceiverAddress,
  ucs03address: AddressCosmosCanonical.pipe(
    // Changed to hex
    Schema.annotations({
      message: () =>
        "ucs03address must be a valid Cosmos Zkgm address in hex (e.g., 0x followed by 40 or 64 hex chars)"
    })
  )
}) {}

export class AptosTransfer extends Schema.Class<AptosTransfer>("AptosTransfer")({
  ...BaseTransferFields,
  sourceRpcType: RpcType.pipe(
    Schema.filter(v => v === "aptos"),
    Schema.annotations({ message: () => "type must be 'aptos'" })
  ),
  wethToken: Schema.Null,
  receiver: ReceiverAddress,
  ucs03address: AddressAptosCanonical.pipe(
    Schema.annotations({
      message: () =>
        "ucs03address must be a valid Aptos Zkgm address (e.g., 0x followed by 64 hex chars)"
    })
  )
}) {}

export const TransferSchema = Schema.Union(EVMTransfer, CosmosTransfer, AptosTransfer).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
})

export type Transfer = Schema.Schema.Type<typeof TransferSchema>

export type EVMTransferType = Schema.Schema.Type<typeof EVMTransfer>
export type CosmosTransferType = Schema.Schema.Type<typeof CosmosTransfer>
export type AptosTransferType = Schema.Schema.Type<typeof AptosTransfer>
