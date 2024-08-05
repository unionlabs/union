import { graphql } from "../index.js"

export const profileTransfersTimestampFilterQueryDocument = graphql(/* graphql */ `
  query AddressesTransfersTimestampFilterQuery(
    $limit: Int!,
    $addresses: [ String! ]!,
    $timestamp: timestamptz!
  ) @cached(ttl: 1000) {
    newer: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: asc },
        { source_transaction_hash: asc }
      ],
      where: {
        _and: [
          { source_timestamp: { _gte: $timestamp } },
          {
            _or: [
              { normalized_sender: { _in: $addresses } },
              { normalized_receiver: { _in: $addresses } }
            ]
          }
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

      forwards {
        port
        channel
        receiver
        chain { chain_id }
      }
    }

  older: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: desc },
        { source_transaction_hash: desc }
      ],
      where: {
        _and: [
          { source_timestamp: { _lt: $timestamp } },
          {
            _or: [
              { normalized_sender: { _in: $addresses } },
              { normalized_receiver: { _in: $addresses } }
            ]
          }
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

      forwards {
        port
        channel
        receiver
        chain { chain_id }
      }
    }
  }
`)

export const latestProfileTransfersQueryDocument = graphql(/* graphql */ `
  query LatestAddressesTransfersQuery(
    $limit: Int!,
    $addresses: [ String! ]!
  ) {
    data: v0_transfers(
      limit: $limit,
      order_by: [
        { source_timestamp: desc },
        { source_transaction_hash: desc }
      ],
      where: {
        _or: [
          { normalized_sender: { _in: $addresses } },
          { normalized_receiver: { _in: $addresses } }
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

      forwards {
        port
        channel
        receiver
        chain { chain_id }
      }
    }
  }
`)
