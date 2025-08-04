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
 * @since 1.0.0
 * @category layers
 */
export const layerXMLZkgmRequest: Layer.Layer<ZkgmClient.ZkgmClient> = internal.layerXMLHttpRequest

/**
 * @since 1.0.0
 * @category tags
 */
export class XMLHttpRequest extends Context.Tag(internal.xhrTagKey)<
  XMLHttpRequest,
  LazyArg<globalThis.XMLHttpRequest>
>() {}

/**
 * @since 1.0.0
 * @category fiber refs
 */
export const currentXHRResponseType: FiberRef.FiberRef<"text" | "arraybuffer"> =
  internal.currentXHRResponseType

/**
 * @since 1.0.0
 * @category fiber refs
 */
export const withXHRArrayBuffer: <A, E, R>(effect: Effect<A, E, R>) => Effect<A, E, R> =
  internal.withXHRArrayBuffer
