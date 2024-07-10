import "temporal-polyfill/global"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
  transfersTimestampFilterQueryDocument,
  transfersBeforeTimestampQueryDocument,
  transfersOnOrAfterTimestampQueryDocument
} from "$lib/graphql/documents/transfers.ts"

export async function latestTransfers({ limit = 12 }: { limit?: number } = {}) {
  const { data } = await request(URLS.GRAPHQL, latestTransfersQueryDocument, { limit })

  return data.map(transfer => ({
    source: {
      chain_display_name: transfer.source_chain?.display_name,
      address: transfer.sender || "unknown"
    },
    destination: {
      chain_display_name: transfer.destination_chain?.display_name,
      address: transfer.receiver || "unknown"
    },
    timestamp: transfer.source_timestamp,
    hash: transfer.source_transaction_hash,
    assets: transfer.assets
  }))
}

export async function transfersBeforeTimestamp({
  limit = 12,
  timestamp
}: {
  limit?: number
  timestamp: string
}) {
  const { data } = await request(URLS.GRAPHQL, transfersBeforeTimestampQueryDocument, {
    limit,
    timestamp
  })

  return data.map(transfer => ({
    source: {
      chain_display_name: transfer.source_chain?.display_name,
      address: transfer.sender || "unknown"
    },
    destination: {
      chain_display_name: transfer.destination_chain?.display_name,
      address: transfer.receiver || "unknown"
    },
    timestamp: transfer.source_timestamp,
    hash: transfer.source_transaction_hash,
    assets: transfer.assets
  }))
}

export async function transfersOnOrAfterTimestamp({
  limit = 12,
  timestamp
}: {
  limit?: number
  timestamp: string
}) {
  const { data } = await request(URLS.GRAPHQL, transfersOnOrAfterTimestampQueryDocument, {
    limit,
    timestamp
  })

  return data.map(transfer => ({
    source: {
      chain_display_name: transfer.source_chain?.display_name,
      address: transfer.sender || "unknown"
    },
    destination: {
      chain_display_name: transfer.destination_chain?.display_name,
      address: transfer.receiver || "unknown"
    },
    timestamp: transfer.source_timestamp,
    hash: transfer.source_transaction_hash,
    assets: transfer.assets
  }))
}

export async function paginatedTransfers({
  limit = 12,
  timestamp
}: { limit?: number; timestamp: string }) {
  const { older, newer } = await request(URLS.GRAPHQL, transfersTimestampFilterQueryDocument, {
    limit,
    timestamp
  })

  const allTransfers = [...newer, ...older].sort(
    // @ts-expect-error
    (a, b) => new Date(b.source_timestamp) - new Date(a.source_timestamp)
  )

  return allTransfers.map(transfer => ({
    source: {
      chain_display_name: transfer.source_chain?.display_name,
      address: transfer.sender || "unknown"
    },
    destination: {
      chain_display_name: transfer.destination_chain?.display_name,
      address: transfer.receiver || "unknown"
    },
    timestamp: transfer.source_timestamp,
    hash: transfer.source_transaction_hash,
    assets: transfer.assets
  }))
}

// paginatedTransfers({ timestamp: "2024-07-10T00:49:00.368143+00:00" }).then(console.log)
