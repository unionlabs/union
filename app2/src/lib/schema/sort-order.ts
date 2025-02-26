import { Schema } from "effect"

export const SortOrder = Schema.String.pipe(Schema.pattern(/^\d{14}-0x[0-9a-fA-F]+$/)).pipe(
  Schema.brand("SortOrder")
)
