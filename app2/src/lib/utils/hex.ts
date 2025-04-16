import { Data, Effect } from "effect"
import { fromHex, type FromHexErrorType } from "viem"

export class FromHexError extends Data.TaggedError("FromHexError")<{
  cause: FromHexErrorType
}> {}

export const fromHexString = (hex: `0x${string}`) =>
  Effect.try({
    try: () => fromHex(hex, "string"),
    catch: error => new FromHexError({ cause: error as FromHexErrorType })
  })
