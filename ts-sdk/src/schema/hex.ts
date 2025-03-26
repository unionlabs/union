import * as S from "effect/Schema"

/**
 * XXX: This will not produce appropriate arbitrary values or protect against
 * invalid input. `TemplateLiteral`s do not support refinements.
 * 
 * False positives include:
 * - "0x"
 * 
 */
// export const Hex = S.TemplateLiteral("0x", S.String).pipe(S.pattern(/^0x[0-9a-f]+$/))

export const Hex = S.NonEmptyString.pipe(
  S.pattern(/^0x[0-9a-f]+$/),
  S.minLength(3)
) as unknown as S.TemplateLiteral<`0x${string}`>

/** XXX: this type signature is a lie */
export type Hex = `0x${string}`

// TODO: validate ERC55 checksum
// XXX: invalid for the above reasoning
export const HexChecksum = S.TemplateLiteral("0x", S.String).pipe(
  S.pattern(/^0x[0-9a-fA-F]+$/)
)
export type HexChecksum = typeof HexChecksum.Type