import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
  transfersTimestampFilterQueryDocument
} from "$lib/graphql/documents/transfers.ts"
import {
  latestAddressTransfersQueryDocument,
  addressTransfersTimestampFilterQueryDocument
} from "$lib/graphql/documents/address-transfers.ts"
import { raise } from "$lib/utilities/index.ts"
import type { PaginatedTransfers } from "./types.ts"

export async function transfersLive({
  limit = 12
}: { limit?: number } = {}): Promise<PaginatedTransfers> {
  const { data } = await request(URLS.GRAPHQL, latestTransfersQueryDocument, { limit })

  return {
    transfers: data.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
        source: {
          hash: transfer.source_transaction_hash || "unknown",
          chainId: transfer.source_chain_id ?? raise("source_chain_id is null"),
          address: transfer.sender || "unknown"
        },
        destination: {
          hash: transfer.destination_transaction_hash || "unknown",
          chainId: destinationChainId ?? raise("destination_chain_id is null"),
          address: receiver || "unknown"
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

export async function transfersByTimestamp({
  limit,
  timestamp
}: { limit: number; timestamp: string }): Promise<PaginatedTransfers> {
  const { older, newer } = await request(URLS.GRAPHQL, transfersTimestampFilterQueryDocument, {
    limit,
    timestamp
  })

  const allTransfers = [...newer.toReversed(), ...older]

  return {
    transfers: allTransfers.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
        source: {
          hash: transfer.source_transaction_hash || "unknown",
          chainId: transfer.source_chain_id ?? raise("source_chain_id is null"),
          address: transfer.sender || "unknown"
        },
        destination: {
          hash: transfer.destination_transaction_hash || "unknown",
          chainId: destinationChainId ?? raise("destination_chain_id is null"),
          address: receiver || "unknown"
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

export async function transfersLiveByAddress({
  limit,
  addresses
}: {
  limit: number
  addresses: Array<string>
}): Promise<PaginatedTransfers> {
  const { data } = await request(URLS.GRAPHQL, latestAddressTransfersQueryDocument, {
    limit,
    addresses
  })

  return {
    transfers: data.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
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

export async function transfersByTimestampForAddresses({
  limit,
  addresses,
  timestamp
}: {
  limit: number
  timestamp: string
  addresses: Array<string>
}): Promise<PaginatedTransfers> {
  const { older, newer } = await request(
    URLS.GRAPHQL,
    addressTransfersTimestampFilterQueryDocument,
    {
      limit,
      timestamp,
      addresses
    }
  )

  const allTransfers = [...newer.toReversed(), ...older]

  return {
    transfers: allTransfers.map(transfer => {
      const lastForward = transfer.forwards?.at(-1)
      const receiver = lastForward?.receiver ?? transfer.receiver
      const destinationChainId = lastForward?.chain?.chain_id ?? transfer.destination_chain_id
      return {
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
