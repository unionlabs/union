import type { Account } from "viem"
import type { EvmChainId } from "./evm.ts"
import type { CosmosChainId } from "./cosmos.ts"
import type { OfflineSigner } from "../types.ts"

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
