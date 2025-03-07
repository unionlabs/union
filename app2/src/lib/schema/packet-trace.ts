import { Schema } from "effect"
import { ChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"

export class PacketTrace extends Schema.Class<PacketTrace>("PacketTrace")({
  type: Schema.String,
  chain: Schema.Struct({
    chain_id: ChainId
  }),
  height: Schema.OptionFromNullOr(Schema.Number),
  block_hash: Schema.OptionFromNullOr(Hex),
  timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  transaction_hash: Schema.OptionFromNullOr(Hex)
}) {}
