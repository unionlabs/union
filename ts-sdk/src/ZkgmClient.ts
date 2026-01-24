/**
 * @since 2.0.0
 */
import type * as Context from "effect/Context"
import type * as Effect from "effect/Effect"
import { RuntimeFiber } from "effect/Fiber"
import type * as FiberRef from "effect/FiberRef"
import type { Inspectable } from "effect/Inspectable"
import { Layer } from "effect/Layer"
import type { Pipeable } from "effect/Pipeable"
import type * as Predicate from "effect/Predicate"
import * as internal from "./internal/zkgmClient.js"
import type * as ClientError from "./ZkgmClientError.js"
import type * as ClientRequest from "./ZkgmClientRequest.js"
import type * as ClientResponse from "./ZkgmClientResponse.js"

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
export interface ZkgmClient extends ZkgmClient.With<ClientError.ClientError> {}

/**
 * @since 2.0.0
 */
export declare namespace ZkgmClient {
  /**
   * @category models
   * @since 2.0.0
   */
  export interface With<E, R = never> extends Pipeable, Inspectable {
    readonly [TypeId]: TypeId
    readonly execute: (
      request: ClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>
  }

  /**
   * @category models
   * @since 2.0.0
   */
  export type Preprocess<E, R> = (
    request: ClientRequest.ZkgmClientRequest,
  ) => Effect.Effect<ClientRequest.ZkgmClientRequest, E, R>

  /**
   * @category models
   * @since 2.0.0
   */
  export type Postprocess<E = never, R = never> = (
    request: Effect.Effect<ClientRequest.ZkgmClientRequest, E, R>,
  ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>
}

/**
 * @category tags
 * @since 2.0.0
 */
export const ZkgmClient: Context.Tag<ZkgmClient, ZkgmClient> = internal.tag

/**
 * @category accessors
 * @since 2.0.0
 */
export const execute: (
  request: ClientRequest.ZkgmClientRequest,
) => Effect.Effect<ClientResponse.ZkgmClientResponse, ClientError.ClientError, ZkgmClient> =
  internal.execute

/**
 * @since 2.0.0
 * @category constructors
 */
export const makeWith: <E2, R2, E, R>(
  postprocess: (
    request: Effect.Effect<ClientRequest.ZkgmClientRequest, E2, R2>,
  ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
  preprocess: ZkgmClient.Preprocess<E2, R2>,
) => ZkgmClient.With<E, R> = internal.makeWith

/**
 * @since 2.0.0
 * @category constructors
 */
export const make: (
  f: (
    request: ClientRequest.ZkgmClientRequest,
    signal: AbortSignal,
    fiber: RuntimeFiber<ClientResponse.ZkgmClientResponse, ClientError.ClientError>,
  ) => Effect.Effect<ClientResponse.ZkgmClientResponse, ClientError.ClientError>,
) => ZkgmClient = internal.make

/**
 * @since 2.0.0
 * @category mapping & sequencing
 */
export const transform: {
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
      request: ClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ): (self: ZkgmClient.With<E, R>) => ZkgmClient.With<E | E1, R | R1>
  <E, R, E1, R1>(
    self: ZkgmClient.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
      request: ClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ): ZkgmClient.With<E | E1, R | R1>
} = internal.transform

/**
 * @since 1.0.0
 * @category mapping & sequencing
 */
export const transformResponse: {
  <E, R, E1, R1>(
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ): (self: ZkgmClient.With<E, R>) => ZkgmClient.With<E1, R1>
  <E, R, E1, R1>(
    self: ZkgmClient.With<E, R>,
    f: (
      effect: Effect.Effect<ClientResponse.ZkgmClientResponse, E, R>,
    ) => Effect.Effect<ClientResponse.ZkgmClientResponse, E1, R1>,
  ): ZkgmClient.With<E1, R1>
} = internal.transformResponse

/**
 * @since 2.0.0-beta.3
 * @category tracing
 */
export interface SpanNameGenerator {
  readonly _: unique symbol
}

/**
 * @since 2.0.0-beta.3
 * @category tracing
 */
export const SpanNameGenerator: Context.Reference<
  SpanNameGenerator,
  (request: ClientRequest.ZkgmClientRequest) => string
> = internal.SpanNameGenerator

/**
 * Customizes the span names for tracing.
 *
 * @since 2.0.0-beta.3
 * @category tracing
 */
export const withSpanNameGenerator: {
  (
    f: (request: ClientRequest.ZkgmClientRequest) => string,
  ): <E, R>(self: ZkgmClient.With<E, R>) => ZkgmClient.With<E, R>
  <E, R>(
    self: ZkgmClient.With<E, R>,
    f: (request: ClientRequest.ZkgmClientRequest) => string,
  ): ZkgmClient.With<E, R>
} = internal.withSpanNameGenerator

/**
 * @since 2.0.0-beta.3
 * @category tracing
 */
export const currentTracerDisabledWhen: FiberRef.FiberRef<
  Predicate.Predicate<ClientRequest.ZkgmClientRequest>
> = internal.currentTracerDisabledWhen

/**
 * Disables tracing for specific requests based on a provided predicate.
 *
 * @since 2.0.0-beta.3
 * @category tracing
 */
export const withTracerDisabledWhen: {
  (
    predicate: Predicate.Predicate<ClientRequest.ZkgmClientRequest>,
  ): <E, R>(self: ZkgmClient.With<E, R>) => ZkgmClient.With<E, R>
  <E, R>(
    self: ZkgmClient.With<E, R>,
    predicate: Predicate.Predicate<ClientRequest.ZkgmClientRequest>,
  ): ZkgmClient.With<E, R>
} = internal.withTracerDisabledWhen

/**
 * @since 2.0.0
 */
export const layerMergedContext: <E, R>(
  effect: Effect.Effect<ZkgmClient, E, R>,
) => Layer<ZkgmClient, E, R> = internal.layerMergedContext
