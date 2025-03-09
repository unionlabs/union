import { Data, Effect } from "effect"

export class Base64EncodeError extends Data.TaggedError("Base64EncodeError")<{
  cause: unknown
}> {}

export const toBase64 = (data: unknown) =>
  Effect.try({
    try: () => btoa(JSON.stringify(data)),
    catch: error => new Base64EncodeError({ cause: error })
  })
