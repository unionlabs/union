import { Schema } from "effect"

export const Hex = Schema.String.pipe(Schema.pattern(/^0x[0-9a-f]+$/))

// TODO: validate ERC55 checksum
export const HexChecksum = Schema.String.pipe(Schema.pattern(/^0x[0-9a-fA-F]+$/))
