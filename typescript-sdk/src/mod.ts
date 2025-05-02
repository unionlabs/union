export {
  bech32AddressToHex,
  bech32ToBech32Address,
  bech32ToBytes,
  bytesToBech32Address,
  bytesToHex,
  hexAddressToBech32,
  hexToBytes,
} from "./convert.ts"
export type * from "./types.ts"
import {
  type AptosBrowserWallet,
  type AptosChainId,
  aptosChainId,
  type AptosClientParameters,
  createAptosClient,
} from "./aptos/client.ts"
import {
  type CosmosChainId,
  cosmosChainId,
  type CosmosClientParameters,
  createCosmosClient,
} from "./cosmos/client.ts"
import {
  arbitrumSepolia,
  berachainTestnetbArtio,
  createEvmClient,
  evmChainFromChainId,
  type EvmChainId,
  evmChainId,
  evmChains,
  type EvmClientParameters,
  scrollSepolia,
  sepolia,
} from "./evm/client.ts"
export { createPfmMemo, getHubbleChainDetails } from "./pfm.ts"
export { offchainQuery } from "./query/offchain/hubble.ts"
export {
  getChannelInfo,
  getQuoteToken,
  getRecommendedChannels,
  getWethQuoteToken,
} from "./query/offchain/ucs03-channels.ts"
export {
  getAptosAccountTransactions,
  getCosmosAccountTransactions,
  getCosmosHeight,
  getCosmosTransactionReceipt,
  queryContractState,
  queryCosmosC20TokenMetadata,
  queryCosmosCW20AddressBalance,
} from "./query/on-chain.ts"
export {
  extractBech32AddressPrefix,
  isValidBech32Address,
  isValidBech32ContractAddress,
  isValidCosmosTxHash,
  isValidEvmAddress,
  isValidEvmTxHash,
  truncateAddress,
} from "./utilities/address.ts"
import type { ChainId, TransferAssetsParametersLegacy } from "./types.ts"

export { fallback, http } from "viem"
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
type AptosClient = ReturnType<typeof createAptosClient>

export const GRAQPHQL_URL = "https://staging.graphql.union.build/v1/graphql"

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
  parameters: EvmClientParameters,
): ReturnType<typeof createEvmClient>

/**
 * @example
 * ```ts
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexToBytes } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexToBytes(PRIVATE_KEY)),
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
  parameters: CosmosClientParameters,
): ReturnType<typeof createCosmosClient>

// TODO(kaancaglan): Change the example when example is actually done.
/**
 * @example
 * ```ts
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexToBytes } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexToBytes(PRIVATE_KEY)),
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
  parameters: AptosClientParameters,
): ReturnType<typeof createAptosClient>

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
 * Create Union Client for EVM, Cosmos, and Aptos
 */
export function createUnionClient(
  parameters: EvmClientParameters | CosmosClientParameters | AptosClientParameters,
):
  | ReturnType<typeof createEvmClient>
  | ReturnType<typeof createCosmosClient>
  | ReturnType<typeof createAptosClient>
{
  if (evmChainId.includes(parameters.chainId)) {
    return createEvmClient(parameters as EvmClientParameters)
  }
  if (cosmosChainId.includes(parameters.chainId)) {
    return createCosmosClient(parameters as CosmosClientParameters)
  }
  if (aptosChainId.includes(parameters.chainId)) {
    return createAptosClient(parameters as AptosClientParameters)
  }
  throw new Error("Invalid chain id")
}

/**
 * @example
 * ```ts
 * import { privateKeyToAccount } from "viem/accounts"
 * import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
 * import { createUnionClient, hexToBytes } from "@union/client"
 *
 * const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
 *   Uint8Array.from(hexToBytes(PRIVATE_KEY)),
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
      [KChainId in TChainId]:
        & (
          | EvmClientParameters
          | CosmosClientParameters
          | AptosClientParameters
        )
        & { chainId: KChainId }
    }[TChainId]
  >,
): {
  [KChainId in TChainId]: KChainId extends EvmChainId ? EvmClient
    : KChainId extends CosmosChainId ? CosmosClient
    : KChainId extends AptosChainId ? AptosClient
    : never
} {
  return parameters.reduce(
    (accumulator, parameter) => {
      // @ts-expect-error
      accumulator[parameter.chainId] = createUnionClient(parameter)
      return accumulator
    },
    {} as {
      [KChainId in TChainId]: KChainId extends EvmChainId ? EvmClient
        : KChainId extends CosmosChainId ? CosmosClient
        : KChainId extends AptosChainId ? AptosClient
        : never
    },
  )
}

export {
  type AptosBrowserWallet,
  type AptosChainId,
  aptosChainId,
  type AptosClientParameters,
  arbitrumSepolia,
  berachainTestnetbArtio,
  type ChainId,
  type CosmosChainId,
  cosmosChainId,
  type CosmosClientParameters,
  evmChainFromChainId,
  type EvmChainId,
  evmChainId,
  evmChains,
  type EvmClientParameters,
  scrollSepolia,
  sepolia,
  type TransferAssetsParametersLegacy as TransferAssetsParameters,
}
