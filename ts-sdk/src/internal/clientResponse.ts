import type * as ClientResponse from "../ClientResponse.js"

/** @internal */
export const TypeId: ClientResponse.TypeId = Symbol.for(
  "@unionlabs/sdk/ClientResponse",
) as ClientResponse.TypeId
