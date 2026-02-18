/**
 * This module defines a concrete {@link ZkgmClient} for EVM source chain usage.
 *
 * @since 0.0.0
 */
import * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import * as ZkgmClientError from "@unionlabs/sdk/ZkgmClientError"
import type * as ZkgmClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as Context from "effect/Context"
import * as Effect from "effect/Effect"
import * as Layer from "effect/Layer"
import type * as Viem from "viem"
import type * as Evm from "./Evm.js"
import * as internal from "./internal/zkgmClient.js"

/**
 * @since 0.0.1
 * @category type ids
 */
export const TypeId = `~@unionlabs/sdk-evm/EvmZkgmClient`

/**
 * @since 0.0.1
 * @category models
 */
export interface EvmZkgmClient extends EvmZkgmClient.With<ZkgmClientError.ClientError> {}

/**
 * @since 0.0.1
 * @category models
 */
export namespace EvmZkgmClient {
  /**
   * @since 0.0.1
   * @category models
   */
  export interface With<E, R = never> extends ZkgmClient.ZkgmClient.With<E, R> {
    /**
     * @since 0.0.1
     * @category type ids
     */
    readonly [TypeId]: typeof TypeId
    /**
     * @since 0.0.1
     * @category utils
     */
    readonly prepareEip1193: (
      request: ZkgmClientRequest.ZkgmClientRequest,
    ) => Effect.Effect<{
      readonly preparedRequest: Viem.RpcTransactionRequest
      readonly packetMetadata: {
        readonly salt: `0x${string}`
        readonly timeoutTimestamp: bigint
      }
    }, ZkgmClientError.RequestError>
  }
}

/**
 * @since 0.0.1
 * @category tags
 */
export const EvmZkgmClient = Context.GenericTag<EvmZkgmClient>(TypeId)

/**
 * Provides the agnostic `ZkgmClient` backed by EVM.
 *
 * @category layers
 * @since 0.0.0
 */
export const layerWithoutWallet: Layer.Layer<
  ZkgmClient.ZkgmClient,
  never,
  Evm.WalletClient | Evm.PublicClient
> = internal.layerWithoutWallet

/**
 * Provides both the agnostic `ZkgmClient` and the specialized `EvmZkgmClient`.
 *
 * @category layers
 * @since 0.0.1
 */
export const layerDual: Layer.Layer<
  EvmZkgmClient | ZkgmClient.ZkgmClient,
  never,
  Evm.PublicClient | Evm.WalletClient
> = Layer.scopedContext(
  Effect.map(internal.makeDual, (client) =>
    Context.make(EvmZkgmClient, client).pipe(
      Context.add(ZkgmClient.ZkgmClient, client),
    )),
)

/**
 * Provides both the agnostic `ZkgmClient` and the specialized `EvmZkgmClient` without
 * implementing any behaviors which otherwise require public or wallet clients, causing
 * the produced clients to have no side-effects.
 *
 * @category layers
 * @since 0.0.3
 */
export const layerPure: Layer.Layer<
  EvmZkgmClient | ZkgmClient.ZkgmClient,
  never,
  never
> = Layer.scopedContext(
  Effect.map(internal.makePure, (client) =>
    Context.make(EvmZkgmClient, client).pipe(
      Context.add(ZkgmClient.ZkgmClient, client),
    )),
)
