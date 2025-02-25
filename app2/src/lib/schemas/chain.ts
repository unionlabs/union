import { Schema } from "effect"

export const Chain = Schema.Struct({
  chain_id: Schema.String,
  display_name: Schema.String,
  addr_prefix: Schema.String
})

export const Chains = Schema.Array(Chain)
