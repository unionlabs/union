import type { Readable } from "svelte/store"

export type Pretty<T> = { [K in keyof T]: T[K] } & {}

export type MaybePromise<TValue> = TValue | Promise<TValue>

export type TypeFromSet<T extends Set<any>> = T extends Set<infer U> ? U : never

export type ExtractParameters<T> = T extends new (..._args: infer P) => any ? P[0] : never

export type AtLeastOne<T, U = T> = T extends any
  ? T | (U & Record<Exclude<keyof U, keyof T>, never>)
  : never

export type SelectOneOrMore<T extends string> = AtLeastOne<Record<T, true>>

export type NoRepetition<U extends string, ResultT extends Array<any> = []> =
  | ResultT
  | {
      [k in U]: NoRepetition<Exclude<U, k>, [k, ...ResultT]>
    }[U]

export type AwaitedReturnType<T extends (...args: any) => any> = ReturnType<T> extends Promise<
  infer U
>
  ? U
  : ReturnType<T>

export type UnwrapReadable<T> = T extends Readable<infer U> ? U : never

export type DiscriminatedUnion<K extends PropertyKey, T extends object> = {
  [P in keyof T]: { [Q in K]: P } & T[P] extends infer U ? { [Q in keyof U]: U[Q] } : never
}[keyof T]

export type LooseAutocomplete<T> = {
  [K in keyof T]: T[K]
} & {
  [K: string]: any
}

export type NonNullable<T> = T extends null | undefined ? never : T

export type TODO = any

export type Nullable<T> = T | null | undefined

export type LowercaseKeys<T> = {
  [K in keyof T as Lowercase<K & string>]: T[K]
}

/** @see https://www.totaltypescript.com/concepts/the-prettify-helper */
export type Prettify<T> = {
  [K in keyof T]: T[K]
} & {}

type FinalType<T> = T extends infer U ? { [K in keyof U]: U[K] } : never

export type Override<T, U extends Partial<Record<keyof T, unknown>>> = FinalType<
  Omit<T, keyof U> & U
>
