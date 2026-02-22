import * as Effect from "effect/Effect"
import * as S from "effect/Schema"
import type * as Call from "../Call.js"
import * as Ucs03Ng from "../Ucs03Ng.js"

/** @internal */
export const TypeId: Call.TypeId = Symbol.for(
  "@unionlabs/sdk/Call",
) as Call.TypeId

export const encodeNg = Effect.fn("Call.encodeNg")((self: Call.Call) =>
  S.validate(Ucs03Ng.CallV0)({
    contract_address: self.contractAddress.address,
    contract_calldata: self.contractCalldata,
    eureka: self.eureka,
    sender: self.sender.address,
  })
)
