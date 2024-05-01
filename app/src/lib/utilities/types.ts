export type LowercaseKeys<T> = {
  [K in keyof T as Lowercase<K & string>]: T[K]
}

/** @see https://www.totaltypescript.com/concepts/the-prettify-helper */
export type Prettify<T> = {
  [K in keyof T]: T[K]
} & {}
