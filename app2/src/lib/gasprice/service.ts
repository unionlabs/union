import { BigDecimal, Brand, Context, Effect, Option as O } from "effect"
import type { GasPriceError } from "./error"

export type AtomicGasPrice = BigDecimal.BigDecimal & Brand.Brand<"AtomicGasPrice">
export const AtomicGasPrice = Brand.nominal<AtomicGasPrice>()

export type BaseGasPrice = BigDecimal.BigDecimal & Brand.Brand<"BaseGasPrice">
export const BaseGasPrice = Brand.nominal<BaseGasPrice>()

/**
 * Normalized gas price.
 * @example
 * * 0.0007 ubbn / gas unit => price in BABY / gas unit
 * * 123 wei / gas unit => 0.000000000000000123 ETH / pre gas unit
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
    readonly of: Effect.Effect<{
      value: AtomicGasPrice
      /**
       * e.g. L1 settlement fee on BOB
       */
      additiveFee: O.Option<AtomicGasPrice>
      minimalDenom: string
      denom: string
      decimals: number
    }, GasPriceError>
  }
}
