import { Effect, Layer } from "effect"
import * as GasPrice from "./service.js"

/**
 * @since 0.0.1
 * @category layers
 */
export const layerGasPrice = (): Layer.Layer<GasPrice.GasPrice, never, never> =>
  Layer.effect(
    GasPrice.GasPrice,
    void 0 as unknown as Effect.Effect<GasPrice.GasPrice.Service>,
  )

/**
 * @since 0.0.1
 * @category layers
 */
export const layer = (): Layer.Layer<GasPrice.GasPrice, never, never> =>
  Layer.mergeAll(layerGasPrice())
