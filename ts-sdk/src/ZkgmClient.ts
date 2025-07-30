/**
 * @since 1.0.0
 */
import type * as Context from "effect/Context"
import type * as Effect from "effect/Effect"
import { RuntimeFiber } from "effect/Fiber"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import type * as ClientError from "./ClientError.js"
import type * as ClientResponse from "./ClientResponse.js"
import * as internal from "./internal/zkgmClient.js"
import { Chain } from "./schema/chain.js"
import type * as ZkgmRequest from "./ZkgmRequest.js"

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
      request: ZkgmRequest.ZkgmRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E, R>
    readonly simulate: (
      request: ZkgmRequest.ZkgmRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E, R>

    readonly send: (
      source: Chain,
      destination: Chain,
      sender: string,
      receiver: string,
      amount: bigint,
      options?: ZkgmRequest.Options.Send,
    ) => Effect.Effect<ClientResponse.ClientResponse, E, R>
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export type Preprocess<E, R> = (
    request: ZkgmRequest.ZkgmRequest,
  ) => Effect.Effect<ZkgmRequest.ZkgmRequest, E, R>

  /**
   * @category models
   * @since 2.0.0
   */
  export type Postprocess<E = never, R = never> = (
    request: Effect.Effect<ZkgmRequest.ZkgmRequest, E, R>,
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
  request: ZkgmRequest.ZkgmRequest,
) => Effect.Effect<ClientResponse.ClientResponse, ClientError.ClientError, Client> =
  internal.execute

/**
 * @since 1.0.0
 * @category constructors
 */
export declare const make: (
  f: (
    request: ZkgmRequest.ZkgmRequest,
    url: URL,
    signal: AbortSignal,
    fiber: RuntimeFiber<ClientResponse.ClientResponse, ClientError.ClientError>,
  ) => Effect.Effect<ClientResponse.ClientResponse, ClientError.ClientError>,
) => Client // = internal.make

/**
 * @since 1.0.0
 * @category mapping & sequencing
 */
export const transform: {
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ClientResponse, E, R>,
      request: ZkgmRequest.ZkgmRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E1, R1>,
  ): (self: Client.With<E, R>) => Client.With<E | E1, R | R1>
  <E, R, E1, R1>(
    self: Client.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ClientResponse, E, R>,
      request: ZkgmRequest.ZkgmRequest,
    ) => Effect.Effect<ClientResponse.ClientResponse, E1, R1>,
  ): Client.With<E | E1, R | R1>
} = internal.transform

/**
 * @since 1.0.0
 * @category accessors
 */
export const send: (
  url: string | URL,
  options?: ZkgmRequest.Options.Send | undefined,
) => Effect.Effect<
  ClientResponse.ClientResponse,
  ClientError.ClientError,
  Client
> = internal.send
