import { bech32, bytes } from "@scure/base"
import { Array as Arr, Data, Effect, Option, Schema, Schema as S } from "effect"
import { dual, flow, pipe } from "effect/Function"
import type { Chain as ViemChain } from "viem"
import { VIEM_CHAINS } from "../constants/viem-chains.js"
import * as Ucs05 from "../Ucs05.js"
import type { AddressCosmosCanonical, AddressCosmosDisplay } from "./address.ts"

export const ChainId = S.String.pipe(S.brand("ChainId"))
// e.g. union.union-testnet-9
// TODO: narrow filter for arbitraries
export const UniversalChainId = S.String.pipe(S.pattern(/^[a-z]+\.[a-z\-0-9]+$/)).pipe(
  S.brand("UniversalChainId"),
)
export type UniversalChainId = typeof UniversalChainId.Type

export const ChainDisplayName = S.String.pipe(S.brand("ChainDisplayName"))

export const RpcType = S.Literal("evm", "cosmos", "aptos")
export type RpcType = typeof RpcType.Type

export class ChainFeatures extends S.Class<ChainFeatures>("ChainFeatures")({
  channel_list: S.Boolean,
  connection_list: S.Boolean,
  index_status: S.Boolean,
  packet_list: S.Boolean,
  transfer_submission: S.Boolean,
  transfer_list: S.Boolean,
}) {}

export class ChainReference extends S.Class<Chain>("ChainReference")({
  universal_chain_id: UniversalChainId,
}) {}

export const RpcProtocolType = S.Literal("rpc", "rest", "grpc")
export type RpcProtocolType = typeof RpcProtocolType.Type

export class Rpc extends S.Class<Rpc>("Rpc")({
  type: RpcProtocolType,
  url: S.String,
}) {}

export class Explorer extends S.Class<Explorer>("Explorer")({
  address_url: S.String,
  block_url: S.String,
  description: S.String,
  display_name: S.String,
  home_url: S.String,
  name: S.String,
  tx_url: S.String,
}) {}

export class Edition extends S.Class<Edition>("Edition")({
  environment: S.String,
  name: S.String,
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

export class Chain extends S.Class<Chain>("Chain")({
  chain_id: ChainId,
  universal_chain_id: UniversalChainId,
  minter_address_display: S.NullishOr(S.String), // string | null
  display_name: ChainDisplayName,
  rpc_type: RpcType,
  addr_prefix: Ucs05.HRP,
  testnet: S.Boolean,
  features: S.Array(ChainFeatures),
  rpcs: S.Array(Rpc),
  explorers: S.Array(Explorer),
  editions: S.NullishOr(S.Array(Edition)),
}) {
  toViemChain(): Option.Option<ViemChain> {
    if (this.rpc_type !== "evm") {
      return Option.none()
    }
    return Option.fromNullable(VIEM_CHAINS.find(vc => `${vc.id}` === this.chain_id))
  }

  toCosmosDisplay(
    address: AddressCosmosCanonical,
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
          prefix: this.addr_prefix,
        }),
    })
  }

  getRpcUrl(type: RpcProtocolType): Option.Option<string> {
    return Option.fromNullable(this.rpcs.find(rpc => rpc.type === type)?.url)
  }

  requireRpcUrl(type: RpcProtocolType): Effect.Effect<string, NoRpcError> {
    return Option.match(this.getRpcUrl(type), {
      onNone: () => Effect.fail(new NoRpcError({ chain: this, type })),
      onSome: Effect.succeed,
    })
  }

  requireRpcUrlAsUrl(type: RpcProtocolType): Effect.Effect<URL, NoRpcError> {
    return pipe(
      this.getRpcUrl(type),
      Option.map(flow(
        Schema.decode(Schema.URL),
      )),
      Effect.flatten,
      // XXX: don't mask error
      Effect.mapError(() => new NoRpcError({ chain: this, type })),
    )
  }

  getDisplayAddress(
    address: AddressCosmosCanonical,
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

export const Chains = S.Array(Chain)
export type Chains = typeof Chains.Type

export const getChain: {
  (universalChainId: UniversalChainId): (chains: Chains) => Option.Option<Chain>
  (chains: Chains, universalChainId: UniversalChainId): Option.Option<Chain>
} = dual(
  2,
  (chains: Chains, universalChainId: UniversalChainId): Option.Option<Chain> =>
    Arr.findFirst(chains, chain => chain.universal_chain_id === universalChainId),
)
