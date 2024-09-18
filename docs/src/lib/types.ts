export type MaybePromise<T> = T | Promise<T>

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P]
}

export type NoRepetition<U extends string, ResultT extends Array<any> = []> =
  | ResultT
  | {
      [k in U]: NoRepetition<Exclude<U, k>, [k, ...ResultT]>
    }[U]
