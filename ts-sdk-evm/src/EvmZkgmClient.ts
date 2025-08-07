/**
 * @since 1.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import type * as Layer from "effect/Layer"
import type * as Evm from "./Evm.js"
import type * as EvmWallet from "./EvmWallet.js"
import * as internal from "./internal/zkgmClient.js"

/**
 * @since 1.0.0
 * @category layers
 */
export const layerWithoutWallet: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  EvmWallet.EvmWallet | Evm.PublicClient
> = internal.layerWithoutWallet
