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
        chainId: transfer.source_chain_id,
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id,
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

  const allTransfers = [...newer.toReversed(), ...older]
  // .sort(
  //   // @ts-expect-error
  //   (a, b) => new Date(b.source_timestamp) - new Date(a.source_timestamp)
  // )

  return {
    transfers: allTransfers.map(transfer => ({
      source: {
        chainId: transfer.source_chain_id,
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id,
        address: transfer.receiver || "unknown"
      },
      timestamp: transfer.source_timestamp,
      hash: transfer.source_transaction_hash,
      assets: transfer.assets
    })),
    latestTimestamp: newer.at(-1)?.source_timestamp,
    oldestTimestamp: older.at(-1)?.source_timestamp
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
        chainId: transfer.source_chain_id,
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id,
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
        chainId: transfer.source_chain_id,
        address: transfer.sender || "unknown"
      },
      destination: {
        chainId: transfer.destination_chain_id,
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

export const encodeTimestampSearchParam = (timestamp: string) =>
  `?timestamp=${toPrettyDateTimeFormat(timestamp)?.replaceAll("-", "").replaceAll(":", "").replaceAll(" ", "")}`

export const decodeTimestampSearchParam = (search: string) =>
  search
    .replace("?timestamp=", "")
    .replace(/(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})/, "$1-$2-$3 $4:$5:$6")
