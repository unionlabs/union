/**
 * @since 1.0.0
 */
import type * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import * as Context from "effect/Context"
import type { Effect } from "effect/Effect"
import type * as FiberRef from "effect/FiberRef"
import type { LazyArg } from "effect/Function"
import type * as Layer from "effect/Layer"
import * as internal from "./internal/zkgmClient.js"

/**
 * @since 2.0.0
 * @category layers
 */
export declare const layer: Layer.Layer<ZkgmClient.ZkgmClient>
