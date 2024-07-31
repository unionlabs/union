import { graphql } from "../index.ts"

export const addressTransfersTimestampFilterQueryDocument = graphql(/* graphql */ `
  query AddressTransfersTimestampFilterQuery(
    $limit: Int!,
    $addresses: [ String! ]!,
    $timestamp: timestamptz!
  ) { 
    newer: v0_transfers(
      limit: $limit,
      order_by: [ { source_timestamp: asc },{ source_transaction_hash: asc } ],
      where: { 
        _and: [ 
          { source_timestamp: { _gte: $timestamp } },
          { _or: [ { sender: { _in: $addresses } }, { receiver: { _in: $addresses } } ] },
         ]
      }

    ) { 
      sender
      source_chain_id
      source_timestamp
      source_transaction_hash
      receiver
      destination_chain_id
      destination_timestamp
      destination_transaction_hash
      assets
    }

  older: v0_transfers(
      limit: $limit,
      order_by: [ { source_timestamp: asc },{ source_transaction_hash: asc } ],
      where: { 
        _and: [ 
          { source_timestamp: { _lt: $timestamp } },
          { _or: [ { sender: { _in: $addresses } }, { receiver: { _in: $addresses } } ] },
         ]
      }

    ) { 
      sender
      source_chain_id
      source_timestamp
      source_transaction_hash
      receiver
      destination_chain_id
      destination_timestamp
      destination_transaction_hash
      assets
    }

  }
`)

export const latestAddressTransfersQueryDocument = graphql(/* graphql */ `
  query LatestAddressTransfersQuery(
    $limit: Int!,
    $addresses: [ String! ]!
  ) { 
    data: v0_transfers(
      limit: $limit,
      order_by: { source_timestamp: desc, source_transaction_hash: desc },
      where: { _or: [ { sender: { _in: $addresses } }, { receiver: { _in: $addresses } } ] }
    ) { 
      sender
      source_chain_id
      source_timestamp
      source_transaction_hash
      receiver
      destination_chain_id
      destination_timestamp
      destination_transaction_hash
      assets
    }
  }
`)
