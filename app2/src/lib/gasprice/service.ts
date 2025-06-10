import { BigDecimal, Brand, Context, Effect } from "effect"
import type { GasPriceError } from "./error"

export type AtomicGasPrice = BigDecimal.BigDecimal & Brand.Brand<"AtomicGasPrice">
export const AtomicGasPrice = Brand.nominal<AtomicGasPrice>()

export type BaseGasPrice = BigDecimal.BigDecimal & Brand.Brand<"BaseGasPrice">
export const BaseGasPrice = Brand.nominal<BaseGasPrice>()

/**
 * @since 0.0.1
 * @category tags
 */
export class GasPrice extends Context.Tag("@unionlabs/app/GasPrice")<
  GasPrice,
  GasPrice.Service
>() {}

/**
 * @since 0.0.1
 * @category models
 */
export declare namespace GasPrice {
  /**
   * @since 0.0.1
   * @category models
   */
  export interface Service {
    readonly of: Effect.Effect<BaseGasPrice, GasPriceError>
  }
}
