/**
 * @since 1.0.0
 */
import type * as Context from "effect/Context"
import type * as Effect from "effect/Effect"
import { RuntimeFiber } from "effect/Fiber"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import type * as ClientError from "./ClientError.js"
import type * as ClientRequest from "./ClientRequest.js"
import type * as ClientResponse from "./ClientResponse.js"
import * as internal from "./internal/client.js"

/**
 * @since 2.0.0
 * @category type ids
 */
export const TypeId: unique symbol = internal.TypeId

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @since 2.0.0
 * @category models
 */
export interface Client extends Client.With<ClientError.ClientError> {}

/**
 * @since 2.0.0
 */
export declare namespace Client {
  /**
   * @category models
   * @since 1.0.0
   */
  export interface With<E, R = never> extends Pipeable, Inspectable {
    readonly [TypeId]: TypeId
    readonly execute: (
      request: ClientRequest.ClientRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E, R>
    readonly simulate: (
      request: ClientRequest.ClientRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E, R>
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export type Preprocess<E, R> = (
    request: ClientRequest.ClientRequest,
  ) => Effect.Effect<ClientRequest.ClientRequest, E, R>

  /**
   * @category models
   * @since 2.0.0
   */
  export type Postprocess<E = never, R = never> = (
    request: Effect.Effect<ClientRequest.ClientRequest, E, R>,
  ) => Effect.Effect<ClientResponse.ClientResponse, E, R>
}

/**
 * @category tags
 * @since 2.0.0
 */
export const Client: Context.Tag<Client, Client> = internal.tag

/**
 * @category accessors
 * @since 2.0.0
 */
export const execute: (
  request: ClientRequest.ClientRequest,
) => Effect.Effect<ClientResponse.ClientResponse, ClientError.ClientError, Client> =
  internal.execute

/**
 * @since 1.0.0
 * @category constructors
 */
export const make: (
  f: (
    request: ClientRequest.ClientRequest,
    url: URL,
    signal: AbortSignal,
    fiber: RuntimeFiber<ClientResponse.ClientResponse, ClientError.ClientError>,
  ) => Effect.Effect<ClientResponse.ClientResponse, ClientError.ClientError>,
) => Client = internal.make
