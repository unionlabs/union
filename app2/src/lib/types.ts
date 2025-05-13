import type { Effect, Exit } from "effect"

/**
 * Transform {@link Effect.Effect} to {@link Exit.Exit}
 */
export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E>
  : never

/**
 * Filter sum type to variants which contain a given property key.
 */
export type HasKey<T, K extends PropertyKey> = T extends any ? K extends keyof T ? T
  : never
  : never

/**
 * Map {@link Exit.Success} onto `exit` property of given type.
 */
type WithExitToSuccess<T> = T extends { exit: Exit.Exit<infer A, infer E> } ? {
    [K in keyof T]: K extends "exit" ? Exit.Success<A, E> : T[K]
  }
  : never

/**
 * Drop first entry in an array.
 */
export type Tail<T extends any[]> = T extends [any, ...infer Rest] ? Rest : []
