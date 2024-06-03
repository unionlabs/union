export type NonNullable<T> = T extends null | undefined ? never : T

export type TODO = any

export type MaybePromise<T> = T | Promise<T>

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
