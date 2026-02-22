import * as Effect from "effect/Effect"
import { pipe } from "effect/Function"
import * as Match from "effect/Match"
import * as S from "effect/Schema"
import type * as Batch from "../Batch.js"
import * as Ucs03Ng from "../Ucs03Ng.js"
import * as call_ from "./call.js"
import * as tokenOrder_ from "./tokenOrder.js"

/** @internal */
export const TypeId: Batch.TypeId = Symbol.for(
  "@unionlabs/sdk/Batch",
) as Batch.TypeId

export const encodeNg: (self: Batch.Batch) => Effect.Effect<Ucs03Ng.Batch, any, any> = Effect.fn(
  "Batch.encodeNg",
)((self: Batch.Batch) =>
  pipe(
    self.instructions,
    Effect.forEach((instruction) =>
      pipe(
        Match.value(instruction),
        Match.tagsExhaustive({
          TokenOrder: tokenOrder_.encodeNg,
          Batch: encodeNg,
          Call: call_.encodeNg,
        }),
      )
    ),
    Effect.flatMap((instructions) =>
      S.validate(Ucs03Ng.BatchV0)({
        instructions,
      })
    ),
  )
)
