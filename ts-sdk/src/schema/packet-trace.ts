import { Option } from "effect"
import * as S from "effect/Schema"
import { RpcType, UniversalChainId } from "./chain.js"
import { Height } from "./height.js"
import { Base58FromHex, Hex } from "./hex.js"

export class PacketTrace extends S.Class<PacketTrace>("PacketTrace")({
  type: S.String,
  chain: S.Struct({
    universal_chain_id: UniversalChainId,
    rpc_type: RpcType,
  }),
  height: S.OptionFromNullOr(Height),
  block_hash: S.OptionFromNullOr(Hex),
  timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  transaction_hash: S.OptionFromNullOr(Hex),
}) {
  getDisplayTransactionHash(): Option.Option<string> {
    if (Option.isNone(this.transaction_hash)) {
      return Option.none()
    }

    if (this.chain.rpc_type === "sui") {
      const formatted = S.decodeSync(Base58FromHex)(this.transaction_hash.value)
      return Option.some(formatted)
    }

    return Option.some(this.transaction_hash.value)
  }
}
