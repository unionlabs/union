import "temporal-polyfill/global"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
  transfersTimestampFilterQueryDocument
} from "$lib/graphql/documents/transfers.ts"
import { raise } from "$lib/utilities/index.ts"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export interface TransferAddress {
  chainId: string
  address: string
}

export interface TransferAsset {
  [symbol: string]: {
    amount: string
    info: {
      denom: string
      chain_id: string
      decimals: number
      logo_uri: string | null
      display_name: string | null
      display_symbol: string | null
    }
  }
}

export interface Transfer {
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

export async function latestTransfers({
  limit = 12
}: { limit?: number } = {}): Promise<PaginatedTransfers> {
  const { data } = await request(URLS.GRAPHQL, latestTransfersQueryDocument, { limit })

  return {
    transfers: data.map(transfer => ({
      source: {
        chainId: transfer.source_chain_id ?? raise("source_chain_id is null"),
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id ?? raise("destination_chain_id is null"),
        address: transfer.receiver || "unknown"
      },
      timestamp: `${transfer.source_timestamp}`,
      hash: `${transfer.source_transaction_hash}`,
      assets: transfer.assets
    })),
    latestTimestamp: data.at(0)?.source_timestamp ?? raise("latestTimestamp is null"),
    oldestTimestamp: data.at(-1)?.source_timestamp ?? raise("oldestTimestamp is null")
  }
}

export async function paginatedTransfers({
  limit,
  timestamp
}: { limit: number; timestamp: string }): Promise<PaginatedTransfers> {
  const { older, newer } = await request(URLS.GRAPHQL, transfersTimestampFilterQueryDocument, {
    limit,
    timestamp
  })

  const allTransfers = [...newer.toReversed(), ...older]

  return {
    transfers: allTransfers.map(transfer => ({
      source: {
        chainId: transfer.source_chain_id ?? raise("source_chain_id is null"),
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id ?? raise("destination_chain_id is null"),
        address: transfer.receiver || "unknown"
      },
      timestamp: `${transfer.source_timestamp}`,
      hash: `${transfer.source_transaction_hash}`,
      assets: transfer.assets
    })),
    latestTimestamp: allTransfers.at(0)?.source_timestamp ?? raise("latestTimestamp is null"),
    oldestTimestamp: allTransfers.at(-1)?.source_timestamp ?? raise("oldestTimestamp is null")
  }
}

export const encodeTimestampSearchParam = (timestamp: string) =>
  `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll("-", "").replaceAll(":", "").replaceAll(" ", "")}`

export const decodeTimestampSearchParam = (search: string) =>
  search
    .replace("?timestamp=", "")
    .replace(/(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/, "$1-$2-$3 $4:$5:$6")
