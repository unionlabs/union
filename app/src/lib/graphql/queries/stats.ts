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
