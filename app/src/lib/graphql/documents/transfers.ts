import { graphql } from "../index.ts"

// export const transfersTimestampFilterQueryDocument = graphql(/* GraphQL */ `
//   query TransfersQueryTimestampFilter(
//     $limit: Int! = 10,
//     $timestamp: timestamptz!
//   ) {
//     top: v0_transfers(
//       limit: $limit,
//       order_by: { source_timestamp: asc },
//       where: { source_timestamp: {_gte: $timestamp} },
//     ) {
//       sender
//       source_chain { chain_id display_name }
//       source_timestamp
//       source_transaction_hash
//       receiver
//       destination_chain { chain_id display_name }
//       destination_timestamp
//       destination_transaction_hash
//       assets
//       forwards_2 {
//         chain { chain_id }
//         port
//         channel
//         receiver
//       }
//     }
//     bottom: v0_transfers(
//       limit: $limit,
//       order_by: { source_timestamp: desc },
//       where: { source_timestamp: {_lt: $timestamp} },
//     ) {
//       sender
//       source_chain { chain_id display_name }
//       source_timestamp
//       source_transaction_hash
//       receiver
//       destination_chain { chain_id display_name }
//       destination_timestamp
//       destination_transaction_hash
//       assets
//       forwards_2 {
//         chain { chain_id }
//         port
//         channel
//         receiver
//       }
//     }
//   }
// `)
export const latestTransfersQueryDocument = graphql(/* GraphQL */ `
  query TransfersQuery(
    $limit: Int! = 8,
  ) @cached(ttl: 5) {
    data: v0_transfers(
      limit: $limit,
      order_by: { source_timestamp: desc },
    ) {
      sender
      source_timestamp
      source_transaction_hash
      source_chain { chain_id display_name }
      receiver
      destination_timestamp
      destination_transaction_hash
      destination_chain { chain_id display_name }
      assets
    }
  }  
`)

export const transfersOnOrAfterTimestampQueryDocument = graphql(/* GraphQL */ `
  query TransfersOnOrAfterTimestampQuery(
    $limit: Int! = 8,
    $timestamp: timestamptz!
  ) @cached(ttl: 5) {
    data: v0_transfers(
      limit: $limit,
      order_by: { source_timestamp: asc },
      where: { source_timestamp: {_gte: $timestamp} },
    ) {
      sender
      source_timestamp
      source_transaction_hash
      source_chain { chain_id display_name }
      receiver
      destination_timestamp
      destination_transaction_hash
      destination_chain { chain_id display_name }
      assets
    }
  }
`)

export const transfersBeforeTimestampQueryDocument = graphql(/* GraphQL */ `
  query TransfersBeforeTimestampQuery(
    $limit: Int! = 10,
    $timestamp: timestamptz!
  ) @cached(ttl: 5) {
    data: v0_transfers(
      limit: $limit,
      order_by: { source_timestamp: desc },
      where: { source_timestamp: {_lte: $timestamp} },
    ) {
      sender
      source_timestamp
      source_transaction_hash
      source_chain { chain_id display_name }
      receiver
      destination_timestamp
      destination_transaction_hash
      destination_chain { chain_id display_name }
      assets
    }
  }
`)

export const allTransfersQueryDocument = graphql(/* GraphQL */ `
    query AllTransfersQuery @cached(ttl: 5) {
    v0_transfers(limit: 25, order_by: {source_timestamp: desc}) {
      sender
      source_chain_id
      source_transaction_hash
      receiver
      destination_chain_id
      assets
      source_timestamp
      forwards_2 {
        chain {
          chain_id
        }
        port
        channel
        receiver
      }
    }
  }
`)

export const userTransfersQueryDocument = graphql(/* Graphql */ `
  query UserTransfersQuery($addr1: String!, $addr2: String!) @cached(ttl: 1) {
    v0_transfers(limit: 500, order_by: {source_timestamp: desc}, where: 
  {_or: [
  	{normalized_sender: {_eq: $addr1}}, 
  	{normalized_receiver: {_eq: $addr1}}, 
  	{normalized_sender: {_eq: $addr2}}, 
    {normalized_receiver: {_eq: $addr2}}
  ]}) {
      sender
      normalized_sender
      source_chain_id
      source_connection_id
      source_channel_id
      receiver
      normalized_receiver
      destination_chain_id
      destination_connection_id
      destination_channel_id
      assets
      source_timestamp
      destination_timestamp
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
