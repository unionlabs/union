import { pipe } from "effect/Function"

export type Struct = {
  a: number
  b: bigint
  c: string
}

declare const TypeId: unique symbol
type TypeId = typeof TypeId

export type Build<
  M extends keyof Struct = never,
  R = never,
> = {
  readonly [TypeId]?: TypeId
  /** Contravariant slot for the missing key union */
  readonly _M: (_: M) => void
  /** Contravariant slot for the *environment* */
  readonly _R: (_: R) => void
}

/** TODO: put these in a namespace */
export type Missing<B> = B extends Build<infer M, any> ? M : never
export type Requirements<B> = B extends Build<any, infer R> ? R : never
export type Complete<R = unknown> = Build<never, R>
export type Incomplete<M extends keyof Struct, R = unknown> = Build<M, R>

/** construct proto w/ typeid */
export declare function make<P extends Partial<Struct>>(
  value: P,
): Build<Exclude<keyof Struct, keyof P>, never>

export type Merge<M extends keyof Struct, R> = [M] extends [never] ? Complete<R>
  : Build<M, R>

export declare function addAWith(options: object): <M extends keyof Struct, R>(
  b: Build<M | "a", R>,
) => Merge<Exclude<M, "a">, R | "adds a">
export declare function addBWith(): <M extends keyof Struct, R>(
  b: Build<M | "b", R>,
) => Merge<Exclude<M, "b">, R | "adds b">
export declare function addCWith(): <M extends keyof Struct, R>(
  b: Build<M | "c", R>,
) => Merge<Exclude<M, "c">, R | "adds c">

const incomplete = make({ c: "string" })
export const complete = pipe(
  incomplete,
  addAWith({ someArg: 0 }),
  addBWith(),
)
