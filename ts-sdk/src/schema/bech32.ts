import * as S from "effect/Schema"

// TODO: Bech32 validation
export const Bech32 = S.TemplateLiteral(S.String, "1", S.String).pipe(
  S.brand("Bech32")
)
export type Bech32 = typeof Bech32.Type
