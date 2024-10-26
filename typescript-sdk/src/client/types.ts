import type { OfflineSigner } from "../types.ts"
import type { evmChainId, EvmChainId } from "./evm.ts"
import type { Account as ViemAccount, Address } from "viem"
import type { aptosChainId, AptosChainId } from "./aptos.ts"
import type { cosmosChainId, CosmosChainId } from "./cosmos.ts"
import type { Account as AptosAccount } from "@aptos-labs/ts-sdk"

export type { evmChainId, EvmChainId, cosmosChainId, CosmosChainId, aptosChainId, AptosChainId }

export type ChainId =
  | (typeof evmChainId)[number]
  | (typeof cosmosChainId)[number]
  | (typeof aptosChainId)[number]

export type TransferAssetsParameters<CHAIN_ID extends EvmChainId | CosmosChainId | AptosChainId> = {
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
        account?: ViemAccount | undefined
        relayContractAddress?: Address
      }
    : CHAIN_ID extends AptosChainId
      ? {
          denomAddress: string
          account?: AptosAccount
          relayContractAddress?: string
          gasPrice?: { amount: string; denom: string }
          simulate?: boolean
        }
      : undefined)
