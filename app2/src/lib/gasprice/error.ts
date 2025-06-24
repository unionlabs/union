import * as S from "effect/Schema"

/**
 * @since 1.0.0
 * @category type ids
 */
export const TypeId: unique symbol = Symbol("@unionlabs/app/GasPriceError")

/**
 * @since 1.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @since 1.0.0
 * @category errors
 */
export class GasPriceError
  extends S.TaggedError<GasPriceError>("@unionlabs/app/GasPriceError")("GasPriceError", {
    module: S.String,
    method: S.String,
    description: S.String,
    cause: S.optional(S.Defect),
  })
{
  /**
   * @since 1.0.0
   */
  readonly [TypeId]: TypeId = TypeId
  /**
   * @since 1.0.0
   */
  get message(): string {
    return `[GasPriceError] ${this.module}.${this.method}: ${this.description}`
  }
}
