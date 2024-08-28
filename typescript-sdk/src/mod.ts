import "./patch.ts"
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
import { createPfmMemo } from "./pfm.ts"
import { cosmosHttp } from "./transport.ts"
import { offchainQuery } from "./query/offchain/hubble.ts"
import { createEvmClient, evmChainId, type EvmClientParameters } from "#client/evm.js"
import { cosmosChainId, createCosmosClient, type CosmosClientParameters } from "#client/cosmos.js"

export {
  /**
   * We export this as a standalone so that it can be used to fetch data that get passed to `createCosmosSdkClient`
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

export type * from "./types.ts"

export function createUnionClient(
  parameters: EvmClientParameters
): ReturnType<typeof createEvmClient>

export function createUnionClient(
  parameters: CosmosClientParameters
): ReturnType<typeof createCosmosClient>

export function createUnionClient(parameters: EvmClientParameters | CosmosClientParameters) {
  if (evmChainId.includes(parameters.chainId)) {
    return createEvmClient(parameters as EvmClientParameters)
  }
  if (cosmosChainId.includes(parameters.chainId)) {
    return createCosmosClient(parameters as CosmosClientParameters)
  }
  throw new Error("Invalid chain id")
}
