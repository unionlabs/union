import "temporal-polyfill/global"
import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
  transfersTimestampFilterQueryDocument,
  transfersAfterOrAtTimestampQueryDocument,
  transfersBeforeOrAtTimestampQueryDocument
} from "$lib/graphql/documents/transfers.ts"
import { toPrettyDateTimeFormat } from "$lib/utilities/date.ts"

export async function latestTransfers({ limit = 12 }: { limit?: number } = {}) {
  const { data } = await request(URLS.GRAPHQL, latestTransfersQueryDocument, { limit })

  return {
    transfers: data.map(transfer => ({
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
    })),
    latestTimestamp: data.at(0)?.source_timestamp,
    oldestTimestamp: data.at(-1)?.source_timestamp,
    hasNewer: false,
    hasOlder: true
  }
}

export async function paginatedTransfers({
  limit,
  timestamp
}: { limit: number; timestamp: string }) {
  const { older, newer } = await request(URLS.GRAPHQL, transfersTimestampFilterQueryDocument, {
    limit,
    timestamp
  })

  console.info("older", older.length)
  console.info("newer", newer.length)

  const allTransfers = [...newer, ...older].sort(
    // @ts-expect-error
    (a, b) => new Date(b.source_timestamp) - new Date(a.source_timestamp)
  )

  console.info(allTransfers.map(t => t.source_timestamp))

  return {
    transfers: allTransfers.map(transfer => ({
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
    })),
    latestTimestamp: allTransfers.at(0)?.source_timestamp,
    oldestTimestamp: allTransfers.at(-1)?.source_timestamp,
    hasNewer: newer.length > 0,
    hasOlder: older.length > 0
  }
}

export async function transfersAfterOrAtTimestamp({
  limit,
  timestamp
}: {
  limit: number
  timestamp: string
}) {
  const { data } = await request(URLS.GRAPHQL, transfersAfterOrAtTimestampQueryDocument, {
    limit,
    timestamp
  })

  data.reverse()

  return {
    transfers: data.map(transfer => ({
      source: {
        // chain_display_name: transfer.source_chain?.display_name,
        address: transfer.sender || "unknown"
      },
      destination: {
        // chain_display_name: transfer.destination_chain?.display_name,
        address: transfer.receiver || "unknown"
      },
      timestamp: transfer.source_timestamp,
      hash: transfer.source_transaction_hash,
      assets: transfer.assets
    })),
    latestTimestamp: data.at(0)?.source_timestamp,
    oldestTimestamp: data.at(-1)?.source_timestamp
    // hasNewer: false,
    // hasOlder: true
  }
}

export async function transfersBeforeOrAtTimestamp({
  limit,
  timestamp
}: {
  limit: number
  timestamp: string
}) {
  const { data } = await request(URLS.GRAPHQL, transfersBeforeOrAtTimestampQueryDocument, {
    limit,
    timestamp
  })

  return {
    transfers: data.map(transfer => ({
      source: {
        // chain_display_name: transfer.source_chain?.display_name,
        address: transfer.sender || "unknown"
      },
      destination: {
        // chain_display_name: transfer.destination_chain?.display_name,
        address: transfer.receiver || "unknown"
      },
      timestamp: transfer.source_timestamp,
      hash: transfer.source_transaction_hash,
      assets: transfer.assets
    })),
    latestTimestamp: data.at(0)?.source_timestamp,
    oldestTimestamp: data.at(-1)?.source_timestamp
  }
}

// paginatedTransfers({
//   timestamp: '2024-07-16T08:28:29.418093+00:00',//currentUtcTimestampWithBuffer(),
//   limit: 5
// }).then(console.info)

export const encodeTimestampSearchParam = (timestamp: string) =>
  `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll("-", "").replaceAll(":", "").replaceAll(" ", "")}`

export const decodeTimestampSearchParam = (search: string) =>
  search
    .replace("?timestamp=", "")
    .replace(/(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/, "$1-$2-$3 $4:$5:$6")
