import type { TransferAsset } from "$lib/types.ts"

export interface TransferAddress {
  hash: string
  chainId: string
  address: string
}

export type Transfer = {
  source: TransferAddress
  destination: TransferAddress
  hash: string
  timestamp: string
  assets: TransferAsset
}

export interface PaginatedTransfers {
  transfers: Array<Transfer>
  latestTimestamp: string
  oldestTimestamp: string
}
