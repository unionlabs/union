import { Schema as S } from "effect"

export const Fees = S.Array(S.Struct({
  action: S.Union(
    S.Literal("PACKET_RECV"),
    S.Literal("PACKET_SEND"),
    S.Literal("PACKET_SEND_LC_UPDATE_L0"),
    S.Literal("PACKET_SEND_LC_UPDATE_L1"),
    S.Literal("PACKET_SEND_LC_UPDATE_L2"),
  ),
  fee: S.Positive,
}))
export type Fees = typeof Fees.Type
