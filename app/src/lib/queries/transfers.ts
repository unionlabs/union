import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  transfersLatestQuery,
  transfersTimestampQuery,
  transfersByAddressesLatestQuery,
  transfersByAddressesTimestampQuery
} from "$lib/graphql/queries/transfers.ts"
import { transferListDataFragment } from "$lib/graphql/fragments/transfers"
import { raise } from "$lib/utilities/index.ts"

import { readFragment, type FragmentOf } from "gql.tada"
import { createQuery, keepPreviousData } from "@tanstack/svelte-query"
import { derived, type Readable } from "svelte/store"

const transferTransform = (tx: FragmentOf<typeof transferListDataFragment>) => {
  const transfer = readFragment(transferListDataFragment, tx)
  const lastForward = transfer.forwards?.at(-1)
  const receiver = lastForward?.receiver ?? transfer.receiver
  const destinationChainId = lastForward?.source_chain?.chain_id ?? transfer.destination_chain_id
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
    tokens: transfer.tokens
  }
}

type TransfersReturnType = Promise<Array<ReturnType<typeof transferTransform>>>

export async function transfersLatest({
  limit = 12
}: { limit?: number } = {}): TransfersReturnType {
  const { data } = await request(URLS.GRAPHQL, transfersLatestQuery, {
    limit
  })
  return data.map(transferTransform)
}

export async function transfersTimestamp({
  limit,
  timestamp
}: {
  limit: number
  timestamp: string
}): TransfersReturnType {
  const { older, newer } = await request(URLS.GRAPHQL, transfersTimestampQuery, {
    limit: limit / 2,
    timestamp
  })
  const allTransfers = [...newer.toReversed(), ...older]
  return allTransfers.map(transferTransform)
}

export async function transfersByAddressesLatest({
  limit,
  addresses
}: {
  limit: number
  addresses: Array<string>
}): TransfersReturnType {
  const { data } = await request(URLS.GRAPHQL, transfersByAddressesLatestQuery, {
    limit,
    addresses
  })
  return data.map(transferTransform)
}

export async function transfersByAddressesTimestamp({
  limit,
  addresses,
  timestamp
}: {
  limit: number
  timestamp: string
  addresses: Array<string>
}): TransfersReturnType {
  const { older, newer } = await request(URLS.GRAPHQL, transfersByAddressesTimestampQuery, {
    limit: limit / 2,
    timestamp,
    addresses
  })

  const allTransfers = [...newer.toReversed(), ...older]
  return allTransfers.map(transferTransform)
}

export const transfersQuery = (
  normalizedAddresses: Array<string> | null,
  timestamp: Readable<string | null>,
  pageSize: number
) =>
  createQuery(
    derived([timestamp], ([$timestamp]) =>
      normalizedAddresses
        ? $timestamp
          ? {
              queryKey: ["transfers", $timestamp, ...normalizedAddresses],
              refetchOnMount: false,
              refetchOnReconnect: false,
              placeholderData: keepPreviousData,
              staleTime: Number.POSITIVE_INFINITY,
              queryFn: async () =>
                await transfersByAddressesTimestamp({
                  limit: pageSize,
                  timestamp: $timestamp as string,
                  addresses: normalizedAddresses
                })
            }
          : {
              queryKey: ["transfers", "latest", ...normalizedAddresses],
              refetchOnMount: true,
              placeholderData: keepPreviousData,
              refetchOnReconnect: true,
              refetchInterval: () => 5_000,
              queryFn: async () =>
                await transfersByAddressesLatest({
                  limit: pageSize,
                  addresses: normalizedAddresses
                })
            }
        : $timestamp
          ? {
              queryKey: ["transfers", $timestamp],
              refetchOnMount: false,
              refetchOnReconnect: false,
              placeholderData: keepPreviousData,
              staleTime: Number.POSITIVE_INFINITY,
              queryFn: async () =>
                await transfersTimestamp({
                  timestamp: $timestamp as string, // otherwise its disabled
                  limit: pageSize
                })
            }
          : {
              queryKey: ["transfers", "latest"],
              refetchOnMount: true,
              placeholderData: keepPreviousData,
              refetchOnReconnect: true,
              refetchInterval: () => 5_000,
              queryFn: async () => await transfersLatest({ limit: pageSize })
            }
    )
  )
