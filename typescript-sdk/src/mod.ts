export const zkgm = "o______o"

import "./patch.ts"
export type * from "./types.ts"
import {
  byteArrayToHex,
  bech32AddressToHex,
  hexAddressToBech32,
  bytesToBech32Address,
  bech32ToBech32Address,
  hexStringToUint8Array,
  uint8ArrayToHexString
} from "./convert.ts"
import {
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  isValidCosmosTxHash,
  isValidBech32Address
} from "./utilities/address.ts"
import {
  evmChainId,
  createEvmClient,
  type EvmChainId,
  type EvmClientParameters
} from "./client/evm.ts"
import { createPfmMemo } from "./pfm.ts"
import { cosmosHttp } from "./transport.ts"
import type { ChainId } from "src/client/types.ts"
import { offchainQuery } from "./query/offchain/hubble.ts"
import { cosmosChainId, createCosmosClient, type CosmosClientParameters } from "./client/cosmos.ts"

type EvmClient = ReturnType<typeof createEvmClient>
type CosmosClient = ReturnType<typeof createCosmosClient>

export function createUnionClient(
  parameters: EvmClientParameters
): ReturnType<typeof createEvmClient>

export function createUnionClient(
  parameters: CosmosClientParameters
): ReturnType<typeof createCosmosClient>

/**
 * TODO: add JSDoc with examples
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
 * TODO: add JSDoc with examples
 */
export function createUnionClients<TChainId extends ChainId>(
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
  /**
   * We export this as a standalone so that it can be used to fetch data that get passed to `createUnionClient`
   */
  cosmosHttp,
  offchainQuery,
  createPfmMemo,
  byteArrayToHex,
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  bech32AddressToHex,
  hexAddressToBech32,
  isValidCosmosTxHash,
  bytesToBech32Address,
  isValidBech32Address,
  bech32ToBech32Address,
  hexStringToUint8Array,
  uint8ArrayToHexString
}
