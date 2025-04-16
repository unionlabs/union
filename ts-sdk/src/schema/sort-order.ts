import * as S from "effect/Schema"

// Timestamp-PacketHash[-Number]
export const SortOrder = S.String.pipe(S.brand("SortOrder"))
export type SortOrder = typeof SortOrder.Type
