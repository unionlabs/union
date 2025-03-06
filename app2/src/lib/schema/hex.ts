import { Schema } from "effect"

export const Hex = Schema.TemplateLiteral("0x", Schema.String).pipe(Schema.pattern(/^0x[0-9a-f]+$/))

// TODO: validate ERC55 checksum
export const HexChecksum = Schema.TemplateLiteral("0x", Schema.String).pipe(
  Schema.pattern(/^0x[0-9a-fA-F]+$/)
)
