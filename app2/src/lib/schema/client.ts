import { Schema } from "effect"

export const ClientId = Schema.Int.pipe(Schema.brand("ClientId"))
