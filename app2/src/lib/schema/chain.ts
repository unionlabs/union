import { Schema } from "effect"

export const ChainId = Schema.String.pipe(Schema.brand("ChainId"))
export const ChainDisplayName = Schema.String.pipe(Schema.brand("ChainDisplayName"))

export class Chain extends Schema.Class<Chain>("Chain")({
  chain_id: ChainId,
  display_name: ChainDisplayName,
  addr_prefix: Schema.String
}) {}

export const Chains = Schema.Array(Chain)
