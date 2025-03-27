import * as S from "effect/Schema"

/**
 * Describes `0x${string}` from non-empty string.
 *
 * Note: To be used ineroperably with `viem`.
 */
export const Hex = S.NonEmptyString.pipe(
  S.pattern(/^0x[0-9a-f]+$/),
  S.minLength(3)
) as unknown as S.TemplateLiteral<`0x${string}`>
export type Hex = typeof Hex.Type

// TODO: validate ERC55 checksum
// TODO: see `Hex` for type hacking to avoid `TemplateLiteral` incongruency
export const HexChecksum = S.TemplateLiteral("0x", S.String).pipe(S.pattern(/^0x[0-9a-fA-F]+$/))
export type HexChecksum = typeof HexChecksum.Type
