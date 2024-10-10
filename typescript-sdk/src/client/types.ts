import type { Account, Address } from "viem"
import type { OfflineSigner } from "../types.ts"
import type { evmChainId, EvmChainId } from "./evm.ts"
import type { cosmosChainId, CosmosChainId } from "./cosmos.ts"
import type { moveChainId, MoveChainId } from "./move.ts" // Import MoveChainId and moveChainId

export type { evmChainId, EvmChainId, cosmosChainId, CosmosChainId, moveChainId, MoveChainId }

export type ChainId =
  | (typeof evmChainId)[number]
  | (typeof cosmosChainId)[number]
  | (typeof moveChainId)[number]

export type TransferAssetsParameters<CHAIN_ID extends EvmChainId | CosmosChainId | MoveChainId> = {
  memo?: string
  amount: bigint
  receiver: string
  autoApprove?: boolean
  destinationChainId: ChainId
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
    : CHAIN_ID extends MoveChainId // Add Move-specific parameters
      ? {
          denomAddress: string // Move will also need a denomAddress for token identification
          account?: `0x${string}` // Define account type (Move accounts)
          relayContractAddress?: string // Optional relay contract address for cross-chain moves
          gasPrice?: { amount: string; denom: string } // Move might also have gas price logic
        }
      : undefined)
