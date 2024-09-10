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
export function createUnionClient(parameters: EvmClientParameters | CosmosClientParameters) {
  if (evmChainId.includes(parameters.chainId)) {
    return createEvmClient(parameters as EvmClientParameters)
  }
  if (cosmosChainId.includes(parameters.chainId)) {
    return createCosmosClient(parameters as CosmosClientParameters)
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
      [KChainId in TChainId]: (EvmClientParameters | CosmosClientParameters) & { chainId: KChainId }
    }[TChainId]
  >
): { [KChainId in TChainId]: KChainId extends EvmChainId ? EvmClient : CosmosClient } {
  return parameters.reduce(
    (accumulator, parameter) => {
      // @ts-expect-error
      accumulator[parameter.chainId] = createUnionClient(parameter)
      return accumulator
    },
    {} as { [KChainId in TChainId]: KChainId extends EvmChainId ? EvmClient : CosmosClient }
  )
}

export {
  evmChains,
  evmChainId,
  type ChainId,
  cosmosChainId,
  type EvmChainId,
  type CosmosChainId,
  evmChainFromChainId,
  type EvmClientParameters,
  type CosmosClientParameters,
  type TransferAssetsParameters,
  sepolia,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
}
