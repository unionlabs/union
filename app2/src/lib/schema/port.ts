import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"

export const PortId = Hex.pipe(Schema.brand("PortId"))
export type PortId = typeof PortId.Type
