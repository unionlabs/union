export type * from "./types.ts"
export {
  bytesToHex,
  bech32AddressToHex,
  hexAddressToBech32,
  bytesToBech32Address,
  bech32ToBech32Address,
  hexStringToUint8Array,
  uint8ArrayToHexString
} from "./convert.ts"
import {
  evmChains,
  evmChainId,
  createEvmClient,
  type EvmChainId,
  evmChainFromChainId,
  type EvmClientParameters,
  sepolia,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
} from "./client/evm.ts"
import {
  cosmosChainId,
  type CosmosChainId,
  createCosmosClient,
  type CosmosClientParameters
} from "./client/cosmos.ts"
import {
  moveChainId,
  type MoveChainId,
  createMoveClient,
  type MoveClientParameters
} from "./client/move.ts"
export {
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  isValidCosmosTxHash,
  isValidBech32Address,
  extractBech32AddressPrefix
} from "./utilities/address.ts"
export { offchainQuery } from "./query/offchain/hubble.ts"
export { createPfmMemo, getHubbleChainDetails } from "./pfm.ts"
import type { ChainId, TransferAssetsParameters } from "./client/types.ts"
export { http, fallback } from "viem"
/**
 * @module
 *
 * Union Labs TypeScript SDK providing utilities for cross-chain transfers and more.
 *
 * @example
 * ```ts
 * import { createUnionClient } from "@union/client"
 * import { privateKeyToAccount } from "viem/accounts"
 *
 * const client = createUnionClient({
 *   chainId: "11155111",
 *   transport: http("https://rpc.sepolia.org"),
 *   account: privateKeyToAccount(`0x${PRIVATE_KEY}`) // or from wagmi configuration
 * })
 * ```
 */

type EvmClient = ReturnType<typeof createEvmClient>
type CosmosClient = ReturnType<typeof createCosmosClient>
type MoveClient = ReturnType<typeof createMoveClient>

/**
 * @example
 * ```ts
 * import { createUnionClient } from "@union/client"
 * import { privateKeyToAccount } from "viem/accounts"
 *
 * const client = createUnionClient({
 *   chainId: "11155111",
 *   transport: http("https://rpc.sepolia.org"),
 *   account: privateKeyToAccount(`0x${PRIVATE_KEY}`) // or from wagmi configuration
 * })
 * ```
 */
export function createUnionClient(
  parameters: EvmClientParameters
): ReturnType<typeof createEvmClient>

/**
 * @example
 * ```ts
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexStringToUint8Array } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
 *   "stride"
 * )
 *
 * const client = createUnionClient({
 *   account: cosmosAccount,
 *   chainId: "stride-internal-1",
 *   transport: http("stride.testnet-1.stridenet.co"),
 * })
 * ```
 */
export function createUnionClient(
  parameters: CosmosClientParameters
): ReturnType<typeof createCosmosClient>

// TODO(kaancaglan): Change the example when example is actually done.
/**
 * @example
 * ```ts
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexStringToUint8Array } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
 *   "stride"
 * )
 *
 * const client = createUnionClient({
 *   account: cosmosAccount,
 *   chainId: "stride-internal-1",
 *   transport: http("stride.testnet-1.stridenet.co"),
 * })
 * ```
 */
export function createUnionClient(
  parameters: MoveClientParameters
): ReturnType<typeof createMoveClient>

/**
 * @example
 * ```ts
 * import { createUnionClient } from "@union/client"
 * import { privateKeyToAccount } from "viem/accounts"
 *
 * const client = createUnionClient({
 *   chainId: "11155111",
 *   transport: http("https://rpc.sepolia.org"),
 *   account: privateKeyToAccount(`0x${PRIVATE_KEY}`) // or from wagmi configuration
 * })
 * ```
 */
/**
 * Create Union Client for EVM, Cosmos, and Move
 */
export function createUnionClient(
  parameters: EvmClientParameters | CosmosClientParameters | MoveClientParameters
):
  | ReturnType<typeof createEvmClient>
  | ReturnType<typeof createCosmosClient>
  | ReturnType<typeof createMoveClient> {
  if (evmChainId.includes(parameters.chainId)) {
    return createEvmClient(parameters as EvmClientParameters)
  }
  if (cosmosChainId.includes(parameters.chainId)) {
    return createCosmosClient(parameters as CosmosClientParameters)
  }
  if (moveChainId.includes(parameters.chainId)) {
    return createMoveClient(parameters as MoveClientParameters)
  }
  throw new Error("Invalid chain id")
}

/**
 * @example
 * ```ts
 * import { privateKeyToAccount } from "viem/accounts"
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexStringToUint8Array } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexStringToUint8Array(PRIVATE_KEY)),
 *   "stride"
 * )
 *
 * const clients = createMultiUnionClient([
 *   {
 *     chainId: "11155111",
 *     transport: http("https://rpc.sepolia.org"),
 *     account: privateKeyToAccount(`0x${PRIVATE_KEY}`) // or from wagmi configuration
 *   },
 *   {
 *     account: cosmosAccount,
 *     chainId: "stride-internal-1",
 *     transport: http("stride.testnet-1.stridenet.co")
 *   }
 * ])
 * ```
 */
export function createMultiUnionClient<TChainId extends ChainId>(
  parameters: Array<
    {
      [KChainId in TChainId]: (
        | EvmClientParameters
        | CosmosClientParameters
        | MoveClientParameters
      ) & { chainId: KChainId }
    }[TChainId]
  >
): {
  [KChainId in TChainId]: KChainId extends EvmChainId
    ? EvmClient
    : KChainId extends CosmosChainId
      ? CosmosClient
      : KChainId extends MoveChainId
        ? MoveClient
        : never
} {
  return parameters.reduce(
    (accumulator, parameter) => {
      // @ts-expect-error
      accumulator[parameter.chainId] = createUnionClient(parameter)
      return accumulator
    },
    {} as {
      [KChainId in TChainId]: KChainId extends EvmChainId
        ? EvmClient
        : KChainId extends CosmosChainId
          ? CosmosClient
          : KChainId extends MoveChainId
            ? MoveClient
            : never
    }
  )
}

export {
  evmChains,
  evmChainId,
  type ChainId,
  cosmosChainId,
  moveChainId,
  type EvmChainId,
  type CosmosChainId,
  type MoveChainId,
  evmChainFromChainId,
  type EvmClientParameters,
  type CosmosClientParameters,
  type MoveClientParameters,
  type TransferAssetsParameters,
  sepolia,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
}
