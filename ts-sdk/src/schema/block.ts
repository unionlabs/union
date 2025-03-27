import * as S from "effect/Schema"

export const Block = S.Struct({
  result: S.Struct({
    block_id: S.Struct({
      hash: S.String
    })
  })
})
export type Block = typeof Block.Type
