import { Schema } from "effect";
import { AddressAptosCanonical, AddressCosmosCanonical, AddressEvmCanonical } from "$lib/schema/address";
import { RpcType } from "$lib/schema/chain";
import { EVMWethToken, TokenRawAmount, TokenRawDenom } from "$lib/schema/token";
import { ChannelId } from "$lib/schema/channel";

const CommonTransferFields = {
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
  })
};

export class EVMTransfer extends Schema.Class<EVMTransfer>("EVMTransfer")({
  type: RpcType.pipe(
    Schema.filter((v) => v === "evm"),
    Schema.annotations({ message: () => "type must be 'evm'" })
  ),
  ...CommonTransferFields,
  wethToken: EVMWethToken,
  receiver: AddressEvmCanonical.pipe(
    Schema.annotations({
      message: () => "receiver must be a valid EVM canonical address (e.g., 0x followed by 40 hex chars)"
    })
  ),
  ucs03address: AddressEvmCanonical.pipe(
    Schema.annotations({
      message: () => "ucs03address must be a valid EVM Zkgm address (e.g., 0x followed by 40 hex chars)"
    })
  )
}) {}

export class CosmosTransfer extends Schema.Class<CosmosTransfer>("CosmosTransfer")({
  type: RpcType.pipe(
    Schema.filter((v) => v === "cosmos"),
    Schema.annotations({ message: () => "type must be 'cosmos'" })
  ),
  ...CommonTransferFields,
  wethToken: Schema.Null,
  receiver: AddressCosmosCanonical.pipe(
    Schema.annotations({
      message: () => "receiver must be a valid Cosmos canonical address (e.g., 0x followed by 40 or 64 hex chars)"
    })
  ),
  ucs03address: AddressCosmosCanonical.pipe( // Changed to hex
    Schema.annotations({
      message: () => "ucs03address must be a valid Cosmos Zkgm address in hex (e.g., 0x followed by 40 or 64 hex chars)"
    })
  )
}) {}

export class AptosTransfer extends Schema.Class<AptosTransfer>("AptosTransfer")({
  type: RpcType.pipe(
    Schema.filter((v) => v === "aptos"),
    Schema.annotations({ message: () => "type must be 'aptos'" })
  ),
  ...CommonTransferFields,
  wethToken: Schema.Null,
  receiver: AddressAptosCanonical.pipe(
    Schema.annotations({
      message: () => "receiver must be a valid Aptos canonical address (e.g., 0x followed by 64 hex chars)"
    })
  ),
  ucs03address: AddressAptosCanonical.pipe(
    Schema.annotations({
      message: () => "ucs03address must be a valid Aptos Zkgm address (e.g., 0x followed by 64 hex chars)"
    })
  )
}) {}

export const TransferSchema = Schema.Union(
  EVMTransfer,
  CosmosTransfer,
  AptosTransfer
).annotations({
  identifier: "Transfer",
  title: "Transfer",
  description: "transfer arguments"
});

export type Transfer = Schema.Schema.Type<typeof TransferSchema>;

export type EVMTransferType = Schema.Schema.Type<typeof EVMTransfer>;
export type CosmosTransferType = Schema.Schema.Type<typeof CosmosTransfer>;
export type AptosTransferType = Schema.Schema.Type<typeof AptosTransfer>;