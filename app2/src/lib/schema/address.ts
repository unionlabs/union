import { Schema } from "effect"

// TODO: Assert 0x prefix
export const AddressNormalized = Schema.String.pipe(Schema.brand("AddressNormalized"))
