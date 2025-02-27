import { Schema } from "effect"

// Timestamp-PacketHash[-Number]
export const SortOrder = Schema.String.pipe(
  Schema.pattern(/^\d{14}-0x[0-9a-f]{64}(?:-\d+)?$/)
).pipe(Schema.brand("SortOrder"))
