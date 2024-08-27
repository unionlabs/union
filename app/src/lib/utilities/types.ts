import type { Readable } from "svelte/store"

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
