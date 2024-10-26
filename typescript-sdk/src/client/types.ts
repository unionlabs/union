import type {
  aptosChainId,
  AptosChainId,
  AptosAccount,
  AptosBrowserWallet,
  AptosPublicAccountInfo
} from "./aptos.ts"
import type { OfflineSigner } from "../types.ts"
import type { evmChainId, EvmChainId } from "./evm.ts"
import type { Account as ViemAccount, Address } from "viem"
import type { cosmosChainId, CosmosChainId } from "./cosmos.ts"

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
        relayContractAddress?: Address
        account?: ViemAccount | undefined
      }
    : CHAIN_ID extends AptosChainId
      ? {
          simulate?: boolean
          denomAddress: string
          account?: AptosAccount
          relayContractAddress?: string
          gasPrice?: { amount: string; denom: string }
        } & (
          | {
              authAccess: "key"
              account?: AptosAccount
            }
          | {
              authAccess: "wallet"
              account?: AptosPublicAccountInfo
              sign: AptosBrowserWallet["signTransaction"]
            }
        )
      : undefined)
