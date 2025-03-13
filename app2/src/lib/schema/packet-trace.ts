import { Schema } from "effect"
import { UniversalChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"
import { Height } from "./height.ts"

export class PacketTrace extends Schema.Class<PacketTrace>("PacketTrace")({
  type: Schema.String,
  chain: Schema.Struct({
    universal_chain_id: UniversalChainId
  }),
  height: Schema.OptionFromNullOr(Height),
  block_hash: Schema.OptionFromNullOr(Hex),
  timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  transaction_hash: Schema.OptionFromNullOr(Hex)
}) {}
