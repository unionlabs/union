import { Schema } from "effect"
import { ChainId, UniversalChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"

export class PacketTrace extends Schema.Class<PacketTrace>("PacketTrace")({
  type: Schema.String,
  chain: Schema.Struct({
    universal_chain_id: UniversalChainId
  }),
  height: Schema.OptionFromNullOr(Schema.Number),
  block_hash: Schema.OptionFromNullOr(Hex),
  timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  transaction_hash: Schema.OptionFromNullOr(Hex)
}) {}
