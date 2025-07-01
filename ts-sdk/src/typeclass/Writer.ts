/**
 * A Haskell-inspired Writer typeclass.
 *
 * @see https://web.cecs.pdx.edu/~mpj/pubs/springschool.html
 * @since 2.0.0
 */
import type * as covariant from "@effect/typeclass/Covariant"
import type * as flatmap from "@effect/typeclass/FlatMap"
import type * as monad from "@effect/typeclass/Monad"
import type * as monoid from "@effect/typeclass/Monoid"
import type * as of from "@effect/typeclass/Of"
import { dual } from "effect/Function"
import type { TypeLambda } from "effect/HKT"

/**
 * @category typeclasses
 * @since 2.0.0
 */
export type Writer<W, A> = readonly [A, W]

interface WriterTypeLambda<W> extends TypeLambda {
  readonly type: Writer<W, this["Target"]>
}

/**
 * `censor f m` is an action that executes the action `m` and applies the function `f` to its output, leaving the return value unchanged.
 *
 * @category utils
 * @since 2.0.0
 */
export const censor = dual(
  2,
  <A, W>(fa: readonly [A, W], f: (w: W) => W): [A, W] => [fa[0], f(fa[1])],
)

/**
 * Map both the return value and output of a computation using the given function.
 *
 * @category utils
 * @since 2.0.0
 */
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

/**
 * @category utils
 * @since 2.0.0
 */
export const getOf = <W>(M: monoid.Monoid<W>): of.Of<WriterTypeLambda<W>> => ({
  of: <A>(a: A) => [a, M.empty],
})

/**
 * @category utils
 * @since 2.0.0
 */
export const getFlatMap = <W>(M: monoid.Monoid<W>): flatmap.FlatMap<WriterTypeLambda<W>> => ({
  flatMap: dual(2, <A, B>(fa: Writer<W, A>, f: (a: A) => Writer<W, B>): Writer<W, B> => {
    const [a, w1] = fa
    const [b, w2] = f(a)
    return [b, M.combine(w1, w2)]
  }),
})

/**
 * @category utils
 * @since 2.0.0
 */
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

/**
 * Derive a `Writer` typeclass from a `Monoid`.
 *
 * @example
 * import * as Writer from "@unionlabs/sdk/typeclass/Writer"
 * import * as StringInstances from "@effect/typeclass/data/String"
 * import * as FlatMap from "@effect/typeclass/FlatMap"
 * import { pipe } from "effect/Function"
 *
 * const composeK = pipe(
 *   StringInstances.Monoid,
 *   Writer.getMonad,
 *   FlatMap.composeK,
 * )
 *
 * const f = (s: string): [number, string] => [s.length, "[length]"]
 * const g = (n: number): [boolean, string] => [n > 3, `[(>) 3 ${n}]`]
 * const h = pipe(f, composeK(g))
 *
 * assert.deepStrictEqual(h(""), [false, "[length][(>) 3 0]"])
 * assert.deepStrictEqual(h("abc"), [false, "[length][(>) 3 3]"])
 * assert.deepStrictEqual(h("abcd"), [true, "[length][(>) 3 4]"])

 * @category utils
 * @since 2.0.0
 */
export const getMonad = <W>(M: monoid.Monoid<W>): monad.Monad<WriterTypeLambda<W>> => ({
  ...getOf(M),
  ...getCovariant(),
  ...getFlatMap(M),
})
