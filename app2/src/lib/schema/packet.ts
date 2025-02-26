import { Schema } from "effect"

export const PacketHash = Schema.String.pipe(Schema.pattern(/^0x[0-9a-f]{64}$/))
