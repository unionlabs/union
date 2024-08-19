import { graphql } from "../index.ts"

export const TransferListDataFragment = graphql(`
  fragment TransferListData on v0_transfers {
    sender
    source_chain_id
    source_timestamp
    source_transaction_hash
    receiver
    destination_chain_id
    destination_timestamp
    destination_transaction_hash
    assets

    forwards {
      port
      channel
      receiver
      chain { chain_id }
    }
  } 
`)

export const transfersTimestampFilterQueryDocument = graphql(
  /* GraphQL */ `
  query TransfersQueryTimestampFilter(
    $limit: Int!,
    $timestamp: timestamptz!
  ) @cached(ttl: 1000) {
    newer: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: asc },
        { source_transaction_hash: asc }
      ],
      where: { source_timestamp: { _gte: $timestamp } },
    ) {
      ...TransferListData
    }
    older: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: desc },
        { source_transaction_hash: desc }
      ],
      where: { source_timestamp: { _lt: $timestamp } },
    ) {
      ...TransferListData
    }
  }
`,
  [TransferListDataFragment]
)

export const latestTransfersQueryDocument = graphql(
  /* GraphQL */ `
  query TransfersQuery(
    $limit: Int! = 8,
  ) {
    data: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: desc },
        { source_transaction_hash: desc }
      ],
    ) {
      ...TransferListData
    }
  }  
`,
  [TransferListDataFragment]
)

export const userTransfersQueryDocument = graphql(/* Graphql */ `
  query UserTransfersQuery($addr1: String!, $addr2: String!) @cached(ttl: 1) {
    v0_transfers(limit: 500, order_by: {source_timestamp: desc}, where: 
  {_or: [
  	{normalized_sender: {_eq: $addr1}}, 
  	{normalized_receiver: {_eq: $addr1}}, 
  	{normalized_sender: {_eq: $addr2}}, 
    {normalized_receiver: {_eq: $addr2}}
  ]}) {
      ...TransferListData
    }
  }
`)

export const transfersBySourceHashBaseQueryDocument = graphql(/* GraphQL */ `
query TransfersBySourceHashBase($source_transaction_hash: String!) @cached(ttl: 1) {
  v0_transfers(where: {source_transaction_hash: {_eq: $source_transaction_hash}}) {
    sender
    normalized_sender
    source_chain_id
    source_connection_id
    source_channel_id
    source_transaction_hash
    receiver
    normalized_receiver
    destination_chain_id
    destination_connection_id
    destination_channel_id
    assets
    source_timestamp
    destination_timestamp
    forwards_2 {
      chain {
        chain_id
      }
      source_connection_id
      source_channel_id
      destination_connection_id
      destination_channel_id
      channel
      receiver
    }
  }
}
`)

export const transfersBySourceHashTracesAndHopsQueryDocument = graphql(/* GraphQL */ `
query TransfersBySourceHashTracesAndHops($source_transaction_hash: String!) @cached(ttl: 1) {
  v0_transfers(where: {source_transaction_hash: {_eq: $source_transaction_hash}}) {
    traces(order_by: {timestamp: asc}) {
      timestamp
      chain {
        chain_id
      }
      type
      transaction_hash
      height
    }
    hop {
      traces(order_by: {timestamp: asc}) {
        timestamp
        chain {
          chain_id
        }
        type
        transaction_hash
        height
      }
    }
  }
}
`)
