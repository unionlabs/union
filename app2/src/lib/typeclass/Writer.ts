import * as FM from "@effect/typeclass/FlatMap"
import * as S from "@effect/typeclass/Semigroup"
import { dual } from "effect/Function"
import * as HKT from "effect/HKT"

interface WriterTypeLambda<L> extends HKT.TypeLambda {
  readonly type: [this["Target"], L]
}

export const fromSemigroup = <L>(
  S: S.Semigroup<L>,
): FM.FlatMap<WriterTypeLambda<L>> => ({
  flatMap: dual(
    2,
    <A, B>(self: [A, L], f: (a: A) => [B, L]): [B, L] => {
      const [a, l1] = self
      const [b, l2] = f(a)
      return [b, S.combine(l1, l2)]
    },
  ),
})
