/**
 * @since 2.0.0
 */
import type * as Cause from "effect/Cause"
import * as Data from "effect/Data"
// import * as Predicate from "effect/Predicate"
import * as Schema from "effect/Schema"
import type { Simplify } from "effect/Types"

/**
 * @since 2.0.0
 * @category type id
 */
export const TypeId: unique symbol = Symbol.for("@unionlabs/sdk/Error")

/**
 * @since 2.0.0
 * @category type id
 */
export type TypeId = typeof TypeId

/**
 * @since 2.0.0
 * @category refinements
 */
// export const isError = (u: unknown): u is PlatformError => Predicate.hasProperty(u, TypeId)

/**
 * @since 2.0.0
 * @category error
 */
export const TypeIdError = <const TypeId extends symbol, const Tag extends string>(
  typeId: TypeId,
  tag: Tag,
): new<A extends Record<string, any>>(
  args: Simplify<A>,
) =>
  & Cause.YieldableError
  & Record<TypeId, TypeId>
  & { readonly _tag: Tag }
  & Readonly<A> =>
{
  class Base extends Data.Error<{}> {
    readonly _tag = tag
  }
  ;(Base.prototype as any)[typeId] = typeId
  ;(Base.prototype as any).name = tag
  return Base as any
}

/**
 * @since 2.0.0
 * @category Models
 */
export const Module = Schema.Literal(
  "Client",
)
