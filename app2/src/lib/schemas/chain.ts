import { Schema } from "effect"

export class Chain extends Schema.Class<Chain>("Chain")({
  chain_id: Schema.String,
  display_name: Schema.String,
  addr_prefix: Schema.String
}) {}

export const Chains = Schema.Array(Chain)
