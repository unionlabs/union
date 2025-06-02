import { Layer } from "effect"
import * as GasPrice from "./service.js"

/**
 * @since 0.0.1
 * @category layers
 */
export const layerGasPrice = (): Layer.Layer<GasPrice.GasPrice, never, never> =>
  Layer.effect(
    GasPrice.GasPrice,
    make(),
  )

/**
 * @since 0.0.1
 * @category layers
 */
export const layer = (): Layer.Layer<GasPrice.GasPrice, never, never> =>
  Layer.mergeAll(layerGasPrice())
