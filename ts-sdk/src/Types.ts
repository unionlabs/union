/**
 * This module contains utility types.
 *
 * @since 2.0.0
 */
import type { Effect, Exit } from "effect"

/**
 * @category utils
 * @since 2.0.0
 */
export type Tail<T extends readonly any[]> = T extends readonly [any, ...infer U] ? U : []

/**
 * Transform {@link Effect.Effect} to {@link Exit.Exit}
 *
 * @category utils
 * @since 2.0.0
 */
export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E>
  : never

/**
 * Filter sum type to variants which contain a given property key.
 *
 * @category utils
 * @since 2.0.0
 */
export type HasKey<T, K extends PropertyKey> = T extends any ? K extends keyof T ? T
  : never
  : never
