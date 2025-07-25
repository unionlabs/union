/**
 * @since 2.0.0
 */
import { hasProperty } from "effect/Predicate"
import type * as ClientRequest from "./ClientRequest.js"
import type * as ClientResponse from "./ClientResponse.js"
import * as Error from "./Error.js"
import * as internal from "./internal/clientError.js"

/**
 * @since 1.0.0
 * @category type id
 */
export const TypeId: unique symbol = internal.TypeId

/**
 * @since 1.0.0
 * @category type id
 */
export type TypeId = typeof TypeId

/**
 * @since 1.0.0
 * @category guards
 */
export const isClientError = (u: unknown): u is ClientError => hasProperty(u, TypeId)

/**
 * @since 1.0.0
 * @category error
 */
export type ClientError = RequestError | ResponseError

/**
 * @since 1.0.0
 * @category error
 */
export class RequestError extends Error.TypeIdError(TypeId, "RequestError")<{
  readonly request: ClientRequest.ClientRequest
  readonly reason: "Transport" | "Encode" | "InvalidUrl"
  readonly cause?: unknown
  readonly description?: string
}> {
  get methodAndUrl() {
    return `${this.request.method}`
  }

  get message() {
    return this.description
      ? `${this.reason}: ${this.description} (${this.methodAndUrl})`
      : `${this.reason} error (${this.methodAndUrl})`
  }
}

/**
 * @since 1.0.0
 * @category error
 */
export class ResponseError extends Error.TypeIdError(TypeId, "ResponseError")<{
  readonly request: ClientRequest.ClientRequest
  readonly response: ClientResponse.ClientResponse
  readonly reason: "StatusCode" | "Decode" | "EmptyBody"
  readonly cause?: unknown
  readonly description?: string
}> {
  get methodAndUrl() {
    return `${this.request.method}`
  }

  get message() {
    const info = `${this.response.status} ${this.methodAndUrl}`
    return this.description
      ? `${this.reason}: ${this.description} (${info})`
      : `${this.reason} error (${info})`
  }
}
