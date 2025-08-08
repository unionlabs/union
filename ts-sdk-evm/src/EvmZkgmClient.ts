/**
 * This module defines a concrete {@link ZkgmClient} for EVM source chain usage.
 *
 * @since 0.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import type * as Layer from "effect/Layer"
import type * as Evm from "./Evm.js"
import * as internal from "./internal/zkgmClient.js"

/**
 * @category layers
 * @since 0.0.0
 */
export const layerWithoutWallet: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  Evm.WalletClient | Evm.PublicClient
> = internal.layerWithoutWallet
