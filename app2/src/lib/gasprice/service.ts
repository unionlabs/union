import type { Chain } from "@unionlabs/sdk/schema"
import { BigDecimal, Context, Effect } from "effect"
import type { GasPriceError } from "./error"

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
    readonly of: Effect.Effect<BigDecimal.BigDecimal, GasPriceError>
  }
}
