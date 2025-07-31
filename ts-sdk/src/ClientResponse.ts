import type { Effect } from "effect"
import type * as ClientError from "./ClientError.js"
import type * as ClientRequest from "./ClientRequest.js"
import * as internal from "./internal/clientResponse.js"
import IncomingMessage from "./TxIncomingMessage.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = internal.TypeId

/**
 * @category type ids
 * @since 2.0.0
 */
export type TypeId = typeof TypeId

/**
 * @category models
 * @since 2.0.0
 */
export interface ClientResponse extends IncomingMessage.IncomingMessage<ClientError.ResponseError> {
  readonly [TypeId]: TypeId
  readonly request: ClientRequest.ClientRequest
  readonly status: number
  // readonly formData: Effect.Effect<FormData, ClientError.ResponseError>
}

export interface TransferResponse<E> extends IncomingMessage.TxIncomingMessage<E> {
  readonly request: Ready
  readonly txHash: string
  readonly status: "Broadcast" | "Confirmed" | "Finalised"
  readonly meta: Readonly<Record<string, unknown>>
}
