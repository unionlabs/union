import { graphql } from "../index.ts"

export const transferListDataFragment = graphql(`
    fragment TransferListData on v1_ibc_union_fungible_asset_orders {
        sender
        source_chain_id
        packet_send_timestamp
        packet_send_transaction_hash
        receiver
        destination_chain_id
        packet_recv_timestamp
        packet_recv_transaction_hash
        base_token
        base_amount
        base_token_symbol
        quote_token
        quote_amount
    }
`)

// tokens {
//     asset {
//         denom
//         chain {
//             chain_id
//             display_name
//         }
//         decimals
//         logo_uri
//         gas_token
//         display_name
//         display_symbol
//     }
//     amount
//     denom
// }
//
//
// forwards {
//     destination_chain_id
//     receiver
// }
