import "temporal-polyfill/global"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import { raise } from "$lib/utilities/index.ts"
import type { TransferAsset } from "$lib/types.ts"
import {
  latestProfileTransfersQueryDocument,
  profileTransfersTimestampFilterQueryDocument
} from "$lib/graphql/documents/profile-transfers.ts"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

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

export async function latestAddressesTransfers({
  limit,
  addresses
}: {
  limit: number
  addresses: Array<string>
}): Promise<PaginatedTransfers> {
  const { data } = await request(URLS.GRAPHQL, latestProfileTransfersQueryDocument, {
    limit,
    addresses
  })

  return {
    transfers: data.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
        forwards: transfer.forwards,
        source: {
          hash: transfer.source_transaction_hash || "unknown",
          chainId: transfer.source_chain_id ?? raise("source_chain_id is null"),
          address: transfer.sender || "unknown"
        },
        destination: {
          address: receiver || "unknown",
          hash: transfer.destination_transaction_hash || "unknown",
          chainId: destinationChainId ?? raise("destination_chain_id is null")
        },
        timestamp: `${transfer.source_timestamp}`,
        hash: `${transfer.source_transaction_hash}`,
        assets: transfer.assets
      }
    }),
    latestTimestamp: data.at(0)?.source_timestamp ?? raise("latestTimestamp is null"),
    oldestTimestamp: data.at(-1)?.source_timestamp ?? raise("oldestTimestamp is null")
  }
}

export async function paginatedAddressesTransfers({
  limit,
  addresses,
  timestamp
}: {
  limit: number
  timestamp: string
  addresses: Array<string>
}): Promise<PaginatedTransfers> {
  const { newer, older } = await request(
    URLS.GRAPHQL,
    profileTransfersTimestampFilterQueryDocument,
    { limit, addresses, timestamp }
  )

  const allTransfers = [...newer.toReversed(), ...older]

  return {
    transfers: allTransfers.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
        forwards: transfer.forwards,
        source: {
          address: transfer.sender || "unknown",
          hash: transfer.source_transaction_hash || "unknown",
          chainId: transfer.source_chain_id ?? raise("source_chain_id is null")
        },
        destination: {
          address: receiver || "unknown",
          hash: transfer.destination_transaction_hash || "unknown",
          chainId: destinationChainId ?? raise("destination_chain_id is null")
        },
        timestamp: `${transfer.source_timestamp}`,
        hash: `${transfer.source_transaction_hash}`,
        assets: transfer.assets
      }
    }),
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
