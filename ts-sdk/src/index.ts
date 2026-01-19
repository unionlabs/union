/**
 * `@unionlabs/sdk`
 *
 * See https://docs.union.build/integrations/typescript/ for additional integration guides and examples.
 *
 * :::caution[Caution]
 * The SDK is undergoing stabilization efforts, starting with the release of `v2.0.0`, to provide a better developer experience. Breaking changes may occur until `v3.0.0`.
 * :::
 *
 * @since 2.0.0
 */

/**
 * This module handles interaction with the [UCS03](https://docs.union.build/ucs/03/) protocol.
 *
 * @since 2.0.0
 */
export * as Ucs03 from "./Ucs03.js"

/**
 * This module handles interaction with the [UCS05](https://docs.union.build/ucs/05/) standard.
 *
 * @since 2.0.0
 */
export * as Ucs05 from "./Ucs05.js"

/**
 * This module provides USD pricing of a given chain's native token.
 *
 * @since 2.0.0
 */
export * as PriceOracle from "./PriceOracle.js"

/**
 * This module handles [Sui](https://sui.io/) related functionality.
 *
 * @since 2.0.0
 */
export * as Gql from "./Gql.js"

/**
 * This module contains generic utilities.
 *
 * @since 2.0.0
 */
export * as Utils from "./Utils.js"

/**
 * This module handles [Aptos](https://aptosfoundation.org/) chain functionality.
 *
 * :::caution[INCOMPLETE]
 * This module is incomplete. Functionality may be partial. Breaking changes may be necessary and regular.
 * :::
 *
 * @since 2.0.0
 */
export * as Aptos from "./Aptos.js"

/**
 * This module contains utility types.
 *
 * @since 2.0.0
 */
export * as Types from "./Types.js"

/**
 * This modules handles chain-agnostic submission of {@link ZkgmClientRequests}s.
 *
 * @since 2.0.0
 */
export * as ZkgmClient from "./ZkgmClient.js"

/**
 * This module prepares cross-chain request submissions.
 *
 * @since 2.0.0
 */
export * as ZkgmClientRequest from "./ZkgmClientRequest.js"

/**
 * This module sources {@link Channel} data.
 *
 * @since 2.0.0
 */
export * as ChannelRegistry from "./ChannelRegistry.js"

/**
 * This module sources {@link Chain} data.
 *
 * @since 2.0.0
 */
export * as ChainRegistry from "./ChainRegistry.js"

/**
 * This module sources {@link Token} data.
 *
 * @since 2.0.0
 */
export * as TokenRegistry from "./TokenRegistry.js"

/**
 * This module defines structured token data.
 *
 * @since 2.0.0
 */
export * as Token from "./Token.js"

/**
 * This module provides a high-level API for UCS03 `TokenOrderV2` instruction construction.
 *
 * @since 2.0.0
 */
export * as TokenOrder from "./TokenOrder.js"

/**
 * This module provides a high-level API for UCS03 `Batch` instruction construction.
 *
 * @since 2.0.0
 */
export * as Batch from "./Batch.js"

/**
 * This module provides a high-level API for UCS03 `Call` instruction construction.
 *
 * @since 2.0.0
 */
export * as Call from "./Call.js"

/**
 * This module describes high-level requirements for UCS03 instruction, prepared for {@link ZkgmClientRequest}.
 *
 * @since 2.0.0
 */
export * as ZkgmInstruction from "./ZkgmInstruction.js"

/**
 * This module describes possible request and response errors from {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
export * as ZkgmClientError from "./ZkgmClientError.js"

/**
 * This module describes a superset of events during finite-state machine execution following {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
export * as ZkgmIncomingMessage from "./ZkgmIncomingMessage.js"

/**
 * This module describes a chain-agnostic response resulting from {@link ZkgmClient} execution.
 *
 * @since 2.0.0
 */
export * as ZkgmClientResponse from "./ZkgmClientResponse.js"

/**
 * This module interfaces with the indexer to source data.
 *
 * @since 2.0.0
 */
export * as Indexer from "./Indexer.js"

/**
 * This module contains constant data.
 *
 * @since 2.0.0
 */
export * as Constants from "./Constants.js"

/**
 * This module handles liquid staking.
 *
 * @since 2.0.0
 */
export * as Staking from "./Staking.js"

/**
 * This module provides a chain-agnostic service for determining gas price.
 *
 * @since 2.0.0
 */
// export * as GasPrice from "./GasPrice.js"
//

/**
 * This module provides a WASM-bound decode/encode functionality.
 *
 * @since 2.0.0
 */
export * as WasmTest from "./WasmTest.js"
