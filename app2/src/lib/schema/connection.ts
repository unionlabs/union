import { Schema } from "effect"

export const ConnectionId = Schema.Int.pipe(Schema.brand("ConnectionId"))
