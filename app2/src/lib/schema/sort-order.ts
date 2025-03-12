import { Schema } from "effect"

// Timestamp-PacketHash[-Number]
export const SortOrder = Schema.String.pipe(Schema.brand("SortOrder"))
