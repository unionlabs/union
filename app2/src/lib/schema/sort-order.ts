import { Schema } from "effect"

// Timestamp-PacketHash
export const SortOrder = Schema.String.pipe(Schema.pattern(/^\d{14}-0x[0-9a-f]{64}$/)).pipe(
  Schema.brand("SortOrder")
)
