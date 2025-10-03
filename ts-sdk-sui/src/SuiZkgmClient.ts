/**
 * This module defines a concrete {@link ZkgmClient} for Sui source chain usage.
 *
 * @since 0.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import type * as Layer from "effect/Layer"
import * as internal from "./internal/zkgmClient.js"
import type * as Sui from "./Sui.js"

/**
 * @category layers
 * @since 0.0.0
 */
export const layerWithoutWallet: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  Sui.WalletClient | Sui.PublicClient
> = internal.layerWithoutWallet
