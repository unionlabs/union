import { Option, Schema } from "effect"

export const ChainId = Schema.String.pipe(Schema.brand("ChainId"))
// e.g. union.union-testnet-9
export const UniversalChainId = Schema.String.pipe(Schema.brand("UniversalChainId")).pipe(Schema.pattern(/^.*\..*$/)) 
export const ChainDisplayName = Schema.String.pipe(Schema.brand("ChainDisplayName"))

export const RpcType = Schema.Union(
  Schema.Literal("evm"),
  Schema.Literal("cosmos"),
  Schema.Literal("aptos")
)

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

export class Chain extends Schema.Class<Chain>("Chain")({
  chain_id: ChainId,
  universal_chain_id: UniversalChainId,
  display_name: ChainDisplayName,
  rpc_type: RpcType,
  addr_prefix: Schema.String,
  testnet: Schema.Boolean,
  features: Schema.Array(ChainFeatures)
}) {}

export const Chains = Schema.Array(Chain)

export const getChain = (
  chains: typeof Chains.Type,
  chainId: typeof ChainId.Type
): Option.Option<Chain> => Option.fromNullable(chains.find(chain => chain.chain_id === chainId))
