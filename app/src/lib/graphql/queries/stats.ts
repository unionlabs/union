import { graphql } from "gql.tada"

export const statsQueryDocument = graphql(/* GraphQL */ `
    query StatsQuery @cached(ttl: 5) {
        v1_ibc_union_statistics {
            name
            value
        }
    }
`)

export const transfersPerDayQueryDocument = graphql(/* GraphQL */ `
    query TransfersPerDay($limit: Int!) @cached(ttl: 60) {
        v1_ibc_union_daily_fungible_asset_orders(limit: $limit, order_by: {day: desc}) {
            count
            day
        }
    }
`)

export const packetsPerDayQueryDocument = graphql(/* GraphQL */ `
    query PacketsPerDay($limit: Int!) {
        v1_ibc_union_daily_packets(limit: $limit, order_by: {day: desc}) {
            count
        }
    }
`)

export const OrderStatsDocument = graphql(/* GraphQL */ `
    query OrderStats($sourceChainId: String!, $destinationChainId: String!) {
        v1_ibc_union_fungible_asset_order_stats_2(
            where: {
                source_chain: { chain_id: { _eq: $sourceChainId } }
                destination_chain: { chain_id: { _eq: $destinationChainId } }
            }
        ) {
            source_chain {
                chain_id
                display_name
            }
            destination_chain {
                chain_id
                display_name
            }
            packet_send_timestamp_from
            secs_until_packet_ack
            secs_until_packet_recv
            secs_until_write_ack
        }
    }
`)
