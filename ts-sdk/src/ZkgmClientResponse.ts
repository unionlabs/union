/**
 * This module describes a superset of events during finite-state machine execution following {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
import * as O from "effect/Option"
import * as internal from "./internal/zkgmClientResponse.js"
import { Hex } from "./schema/hex.js"
import type * as ClientError from "./ZkgmClientError.js"
import type * as ClientRequest from "./ZkgmClientRequest.js"
import * as IncomingMessage from "./ZkgmIncomingMessage.js"

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
export interface ZkgmClientResponse
  extends IncomingMessage.ZkgmIncomingMessage<ClientError.ResponseError>
{
  readonly [TypeId]: TypeId
  readonly request: ClientRequest.ZkgmClientRequest
  readonly txHash: Hex
  readonly safeHash: O.Option<Hex>
}
