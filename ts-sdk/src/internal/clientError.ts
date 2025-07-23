import type * as Error from "../ClientError.js"

/** @internal */
export const TypeId: Error.TypeId = Symbol.for(
  "@unionlabs/sdk/ClientError",
) as Error.TypeId
