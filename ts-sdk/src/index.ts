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
 * :::caution[INCOMPLETE]
 * This module is incomplete. Functionality may be partial. Breaking changes may be necessary and regular.
 * :::
 *
 * @since 2.0.0
 */
export * as Sui from "./Sui.js"

/**
 * This module handles EVM related functionality.
 *
 * @since 2.0.0
 */
export * as Evm from "./Evm.js"

/**
 * This module handles Cosmos related functionality.
 *
 * @since 2.0.0
 */
export * as Cosmos from "./Cosmos.js"

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
 * This module constructs fungible asset orders for given chains.
 *
 * @since 2.0.0
 */
export * as FungibleAssetOrder from "./FungibleAssetOrder.js"
