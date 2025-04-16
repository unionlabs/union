import * as S from "effect/Schema"
import { UniversalChainId } from "./chain.js"
import { Hex } from "./hex.js"
import { Height } from "./height.js"

export class PacketTrace extends S.Class<PacketTrace>("PacketTrace")({
  type: S.String,
  chain: S.Struct({
    universal_chain_id: UniversalChainId
  }),
  height: S.OptionFromNullOr(Height),
  block_hash: S.OptionFromNullOr(Hex),
  timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  transaction_hash: S.OptionFromNullOr(Hex)
}) {}
