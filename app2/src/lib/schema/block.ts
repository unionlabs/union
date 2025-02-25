import { Schema } from "effect"

export const Block = Schema.Struct({
  result: Schema.Struct({
    block_id: Schema.Struct({
      hash: Schema.String
    })
  })
})
