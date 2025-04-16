import * as S from "effect/Schema"

export const ClientId = S.Int.pipe(S.brand("ClientId"))
export type ClientId = typeof ClientId.Type
