import type { Account, Address } from "viem"
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
  sourceChannel?: string
  destinationChainId: ChainId | (string & {})
} & (CHAIN_ID extends CosmosChainId
  ? {
      denomAddress: string
      account?: OfflineSigner
      relayContractAddress?: string
      gasPrice?: { amount: string; denom: string }
    }
  : CHAIN_ID extends EvmChainId
    ? {
        denomAddress: Address
        relayContractAddress?: Address
        account?: `0x${string}` | Account | undefined
      }
    : undefined)
