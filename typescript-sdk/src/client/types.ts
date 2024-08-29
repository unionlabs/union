import type { Account } from "viem"
import type { OfflineSigner } from "../types.ts"
import type { evmChainId, EvmChainId } from "./evm.ts"
import type { cosmosChainId, CosmosChainId } from "./cosmos.ts"

export type ChainId = (typeof evmChainId)[number] | (typeof cosmosChainId)[number]

export type TransferAssetsParameters<CHAIN_ID extends EvmChainId | CosmosChainId> = {
  memo?: string
  amount: bigint
  recipient: string
  approve?: boolean
  sourcePort?: string
  denomAddress: string
  path: [string, string]
  sourceChannel?: string
  relayContractAddress?: string
} & (CHAIN_ID extends CosmosChainId
  ? { account?: OfflineSigner; gasPrice?: { amount: string; denom: string } }
  : CHAIN_ID extends EvmChainId
    ? { account?: `0x${string}` | Account | undefined }
    : undefined)
