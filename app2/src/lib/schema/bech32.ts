import { Schema } from "effect"

// TODO: Bech32 validation
export const Bech32 = Schema.TemplateLiteral(Schema.String, "1", Schema.String).pipe(
  Schema.brand("Bech32")
)
