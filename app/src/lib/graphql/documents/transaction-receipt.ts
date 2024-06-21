import { graphql } from "gql.tada"

export const transactionReceiptQueryDocument = graphql(/* GraphQL */ `
  query TransactionQuery($hash: String!) {
    data: v0_transfers(
      where: {
      _or: [
        {source_transaction_hash:{_eq: $hash}},
        {destination_transaction_hash:{_eq: $hash}},
      ]
    }) {
      sender
      normalized_sender
      source_chain_id
      source_timestamp
      source_transaction_hash
      
      receiver
      normalized_receiver
      destination_chain_id
      destination_timestamp
      destination_transaction_hash

      source_chain { display_name }
      destination_chain { display_name }

      forwards {
        port
        channel
        receiver
        chain {
          chain_id
        }
      }
    }
  }
`)
