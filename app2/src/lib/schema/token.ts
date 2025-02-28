import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
export const TokenRawAmount = Schema.BigInt.pipe(Schema.brand("TokenRawAmount"))
