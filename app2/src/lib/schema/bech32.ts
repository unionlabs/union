import { Schema } from "effect"

// TODO: Bech32 validation
export const Bech32 = Schema.String.pipe(Schema.brand("Bech32"))
