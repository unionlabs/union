import "temporal-polyfill/global"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
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

// paginatedTransfers({
//   // timestamp: "2024-07-07T15:54:48+00:00"
//   // timestamp: "2024-07-07T15:55:24+00:00"
//   timestamp: Temporal.Now.plainDateTimeISO().toString()
// }).then(_ => console.info(JSON.stringify(_, undefined, 2)))
