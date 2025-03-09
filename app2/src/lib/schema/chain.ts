import { VIEM_CHAINS } from "$lib/constants/viem-chains"
import { Option, Schema } from "effect"
import type { Chain as ViemChain } from "viem"
import type { AddressCosmosCanonical, AddressCosmosDisplay } from "./address"
import { bech32, bytes } from "@scure/base"

export const ChainId = Schema.String.pipe(Schema.brand("ChainId"))
// e.g. union.union-testnet-9
export const UniversalChainId = Schema.String.pipe(Schema.brand("UniversalChainId")).pipe(
  Schema.pattern(/^[^:]+\.[^:]+$/)
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
  chain_id: ChainId,
  universal_chain_id: UniversalChainId
}) {}

export const RpcProtocolType = Schema.Literal("rpc", "rest", "grpc")
export type RpcProtocolType = typeof RpcProtocolType.Type

export class Rpc extends Schema.Class<Rpc>("Rpc")({
  type: RpcProtocolType,
  url: Schema.URL
}) {}

export class Explorer extends Schema.Class<Explorer>("Explorer")({
  address_url: Schema.URL,
  block_url: Schema.URL,
  description: Schema.String,
  display_name: Schema.String,
  home_url: Schema.URL,
  name: Schema.String,
  tx_url: Schema.URL
}) {}

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

  toCosmosDisplay(address: AddressCosmosCanonical): Option.Option<AddressCosmosDisplay> {
    if (this.rpc_type !== "cosmos") {
      return Option.none()
    }

    try {
      const words = bech32.toWords(bytes("hex", address.slice(2)))
      const encoded = bech32.encode(this.addr_prefix, words)
      return Option.some(encoded as AddressCosmosDisplay)
    } catch {
      return Option.none()
    }
  }

  getRpcUrl(type: RpcProtocolType): Option.Option<URL> {
    return Option.fromNullable(this.rpcs.find(rpc => rpc.type === type)?.url)
  }
}

export const Chains = Schema.Array(Chain)

export const getChain = (
  chains: typeof Chains.Type,
  chainId: typeof ChainId.Type
): Option.Option<Chain> => Option.fromNullable(chains.find(chain => chain.chain_id === chainId))
