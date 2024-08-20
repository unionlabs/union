import { graphql } from "../index.ts"

export const transferListDataFragment = graphql(`
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
      chain {
        chain_id
      }
    }
  }
`)
