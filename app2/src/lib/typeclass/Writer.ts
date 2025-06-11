import type * as covariant from "@effect/typeclass/Covariant"
import type * as flatmap from "@effect/typeclass/FlatMap"
import type * as monad from "@effect/typeclass/Monad"
import type * as monoid from "@effect/typeclass/Monoid"
import type * as of from "@effect/typeclass/Of"
import { dual } from "effect/Function"
import type { TypeLambda } from "effect/HKT"

export type Writer<W, A> = readonly [A, W]

interface WriterTypeLambda<W> extends TypeLambda {
  readonly type: Writer<W, this["Target"]>
}

export const censor = dual(
  2,
  <A, W>(fa: readonly [A, W], f: (w: W) => W): [A, W] => [fa[0], f(fa[1])],
)

export const mapWriter: {
  <W1, W2, A, B>(
    f: (a: A, w: W1) => readonly [B, W2],
  ): (writer: readonly [A, W1]) => readonly [B, W2]
  <W1, W2, A, B>(
    writer: readonly [A, W1],
    f: (a: A, w: W1) => readonly [B, W2],
  ): readonly [B, W2]
} = dual(
  2,
  <W1, W2, A, B>(
    writer: readonly [A, W1],
    f: (a: A, w: W1) => readonly [B, W2],
  ): readonly [B, W2] => f(writer[0], writer[1]),
)

export const getOf = <W>(M: monoid.Monoid<W>): of.Of<WriterTypeLambda<W>> => ({
  of: <A>(a: A) => [a, M.empty],
})

export const getFlatMap = <W>(M: monoid.Monoid<W>): flatmap.FlatMap<WriterTypeLambda<W>> => ({
  flatMap: dual(2, <A, B>(fa: Writer<W, A>, f: (a: A) => Writer<W, B>): Writer<W, B> => {
    const [a, w1] = fa
    const [b, w2] = f(a)
    return [b, M.combine(w1, w2)]
  }),
})

export const getCovariant = <W>(): covariant.Covariant<WriterTypeLambda<W>> => ({
  map: dual(2, <A, B>(fa: Writer<W, A>, f: (a: A) => B): Writer<W, B> => [f(fa[0]), fa[1]]),
  imap: dual(
    3,
    <A, B>(
      fa: Writer<W, A>,
      to: (a: A) => B,
      _from: (b: B) => A,
    ): Writer<W, B> => [to(fa[0]), fa[1]],
  ),
})

export const getMonad = <W>(M: monoid.Monoid<W>): monad.Monad<WriterTypeLambda<W>> => ({
  ...getOf(M),
  ...getCovariant(),
  ...getFlatMap(M),
})
