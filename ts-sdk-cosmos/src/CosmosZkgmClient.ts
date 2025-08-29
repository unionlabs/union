/**
 * This module defines a concrete {@link ZkgmClient} for EVM source chain usage.
 *
 * @since 0.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import type { Effect } from "effect"
import type * as Layer from "effect/Layer"
import type * as Cosmos from "./Cosmos.js"
import * as internal from "./internal/zkgmClient.js"

/**
 * @category constructors
 * @since 2.0.0
 */
export const make: Effect.Effect<
  ZkgmClient.ZkgmClient,
  never,
  Cosmos.Client | Cosmos.SigningClient
> = internal.make

/**
 * @category layers
 * @since 0.0.0
 */
export const layerWithoutSigningClient: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  Cosmos.SigningClient | Cosmos.Client
> = internal.layerWithoutSigningClient
