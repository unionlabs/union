import { graphql } from "../index.ts"

export const transferListDataFragment = graphql(`
    fragment TransferListData on v1_transfers {
        sender
        source_chain_id
        source_timestamp
        source_transaction_hash
        receiver
        destination_chain_id
        destination_timestamp
        destination_transaction_hash
        tokens {
            asset {
                denom
                chain {
                    chain_id
                    display_name
                }
                decimals
                logo_uri
                gas_token
                display_name
                display_symbol
            }
            amount
            denom
        }
        forwards {
            source_port_id
            source_channel_id
            receiver
            source_chain {
                chain_id
            }
        }
    }
`)
