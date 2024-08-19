import request from "graphql-request"
import { URLS } from "$lib/constants"
import {
  latestTransfersQueryDocument,
  transfersTimestampFilterQueryDocument,
  TransferListDataFragment
} from "$lib/graphql/documents/transfers.ts"
import {
  latestAddressTransfersQueryDocument,
  addressTransfersTimestampFilterQueryDocument
} from "$lib/graphql/documents/address-transfers.ts"
import { raise } from "$lib/utilities/index.ts"

import { readFragment, type FragmentOf } from "gql.tada"

const transferTransform = (tx: FragmentOf<typeof TransferListDataFragment>) => {
  const transfer = readFragment(TransferListDataFragment, tx)
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
}

export async function transfersLive({ limit = 12 }: { limit?: number } = {}) {
  const { data } = await request(URLS.GRAPHQL, latestTransfersQueryDocument, { limit })

  return data.map(transferTransform)
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
  return allTransfers.map(transferTransform)
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
  return data.map(transferTransform)
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

  return allTransfers.map(transferTransform)
}
