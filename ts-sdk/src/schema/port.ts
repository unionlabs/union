import * as S from "effect/Schema"
import { Hex } from "./hex.js"

export const PortId = Hex.pipe(S.brand("PortId"))
export type PortId = typeof PortId.Type
