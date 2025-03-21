import { VIEM_CHAINS } from "$lib/constants/viem-chains"
import { Data, Effect, Option, Schema } from "effect"
import type { Chain as ViemChain } from "viem"
import type {
  AddressCanonicalBytes,
  AddressCosmosCanonical,
  AddressCosmosDisplay
} from "./address.ts"
import { bech32, bytes } from "@scure/base"

export const ChainId = Schema.String.pipe(Schema.brand("ChainId"))
// e.g. union.union-testnet-9
export const UniversalChainId = Schema.String.pipe(Schema.pattern(/^[^:]+\.[^:]+$/)).pipe(
  Schema.brand("UniversalChainId")
)
export type UniversalChainId = typeof UniversalChainId.Type

export const ChainDisplayName = Schema.String.pipe(Schema.brand("ChainDisplayName"))

export const RpcType = Schema.Literal("evm", "cosmos", "aptos")

export class ChainFeatures extends Schema.Class<ChainFeatures>("ChainFeatures")({
  channel_list: Schema.Boolean,
  connection_list: Schema.Boolean,
  index_status: Schema.Boolean,
  packet_list: Schema.Boolean,
  transfer_submission: Schema.Boolean,
  transfer_list: Schema.Boolean
}) {}

export class ChainReference extends Schema.Class<Chain>("ChainReference")({
  universal_chain_id: UniversalChainId
}) {}

export const RpcProtocolType = Schema.Literal("rpc", "rest", "grpc")
export type RpcProtocolType = typeof RpcProtocolType.Type

export class Rpc extends Schema.Class<Rpc>("Rpc")({
  type: RpcProtocolType,
  url: Schema.String
}) {}

export class Explorer extends Schema.Class<Explorer>("Explorer")({
  address_url: Schema.String,
  block_url: Schema.String,
  description: Schema.String,
  display_name: Schema.String,
  home_url: Schema.String,
  name: Schema.String,
  tx_url: Schema.String
}) {}

export class NoRpcError extends Data.TaggedError("NoRpcError")<{
  chain: Chain
  type: RpcProtocolType
}> {}

export class NotACosmosChainError extends Data.TaggedError("NotACosmosChainError")<{
  chain: Chain
}> {}

export class CosmosAddressEncodeError extends Data.TaggedError("CosmosAddressEncodeError")<{
  cause: unknown
  address: string
  prefix: string
}> {}

export class Chain extends Schema.Class<Chain>("Chain")({
  chain_id: ChainId,
  universal_chain_id: UniversalChainId,
  display_name: ChainDisplayName,
  rpc_type: RpcType,
  addr_prefix: Schema.String,
  testnet: Schema.Boolean,
  features: Schema.Array(ChainFeatures),
  rpcs: Schema.Array(Rpc),
  explorers: Schema.Array(Explorer)
}) {
  toViemChain(): Option.Option<ViemChain> {
    if (this.rpc_type !== "evm") {
      return Option.none()
    }
    return Option.fromNullable(VIEM_CHAINS.find(vc => `${vc.id}` === this.chain_id))
  }

  toCosmosDisplay(
    address: AddressCosmosCanonical
  ): Effect.Effect<AddressCosmosDisplay, NotACosmosChainError | CosmosAddressEncodeError> {
    if (this.rpc_type !== "cosmos") {
      return Effect.fail(new NotACosmosChainError({ chain: this }))
    }

    return Effect.try({
      try: () => {
        const words = bech32.toWords(bytes("hex", address.slice(2)))
        const encoded = bech32.encode(this.addr_prefix, words)
        return encoded as AddressCosmosDisplay
      },
      catch: error =>
        new CosmosAddressEncodeError({
          cause: error,
          address: address,
          prefix: this.addr_prefix
        })
    })
  }

  getRpcUrl(type: RpcProtocolType): Option.Option<URL> {
    return Option.fromNullable(this.rpcs.find(rpc => rpc.type === type)?.url)
  }

  requireRpcUrl(type: RpcProtocolType): Effect.Effect<URL, NoRpcError> {
    return Option.match(this.getRpcUrl(type), {
      onNone: () => Effect.fail(new NoRpcError({ chain: this, type })),
      onSome: Effect.succeed
    })
  }

  getDisplayAddress(
    address: AddressCanonicalBytes
  ): Effect.Effect<string, NotACosmosChainError | CosmosAddressEncodeError> {
    switch (this.rpc_type) {
      case "cosmos":
        return this.toCosmosDisplay(address)
      case "evm":
        // For EVM, capitalize the address
        return Effect.succeed(address.slice(0, 2) + address.slice(2).toUpperCase())
      case "aptos":
        // Aptos uses the canonical format
        return Effect.succeed(address)
      default:
        return Effect.fail(new NotACosmosChainError({ chain: this }))
    }
  }
}

export const Chains = Schema.Array(Chain)
export type Chains = typeof Chains.Type

export const getChain = (
  chains: typeof Chains.Type,
  universalChainId: UniversalChainId
): Option.Option<Chain> =>
  Option.fromNullable(chains.find(chain => chain.universal_chain_id === universalChainId))
