import { graphql } from "gql.tada"

export const transferCountQueryDocument = graphql(/* GraphQL */ `
  query TransferCountQuery @cached(ttl: 5)  {
    v0_transfers_aggregate {
    	aggregate {
      	count
      }
    }
  }
`)

export const packetCountQueryDocument = graphql(/* GraphQL */ `
  query PacketCountQuery @cached(ttl: 5)  {
    v0_packets_aggregate {
    	aggregate {
    	   count
      }
    }
  }
`)

export const transfersPerDayQueryDocument = graphql(/* GraphQL */ `
  query TransfersPerDay($limit: Int!) @cached(ttl: 60) {
    v0_daily_transfers(limit: $limit, order_by: {day: desc}) {
      count
      day
    }
  }
`)

export const packetsPerDayQueryDocument = graphql(/* GraphQL */ `
  query PacketsPerDay($limit: Int!) {
    v0_daily_packets(limit: $limit, order_by: {day: desc}) {
      count
    }
  }
`)
