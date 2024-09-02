import type { Account, Address } from "viem"
import type { OfflineSigner } from "../types.ts"
import type { evmChainId, EvmChainId } from "./evm.ts"
import type { cosmosChainId, CosmosChainId } from "./cosmos.ts"

export type { evmChainId, EvmChainId, cosmosChainId, CosmosChainId }

export type ChainId = (typeof evmChainId)[number] | (typeof cosmosChainId)[number]

export type TransferAssetsParameters<CHAIN_ID extends EvmChainId | CosmosChainId> = {
  memo?: string
  amount: bigint
  recipient: string
  autoApprove?: boolean
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
        simulate?: boolean
        denomAddress: Address
        account?: Account | undefined
        relayContractAddress?: Address
      }
    : undefined)
