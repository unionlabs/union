/**
 * @since 1.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import type * as Layer from "effect/Layer"
import type * as Evm from "./Evm.js"
import * as internal from "./internal/zkgmClient.js"

/**
 * @since 1.0.0
 * @category layers
 */
export const layerWithoutWallet: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  Evm.WalletClient | Evm.PublicClient
> = internal.layerWithoutWallet
