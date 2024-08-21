import { graphql } from "gql.tada"
import { packetListDataFragment } from "$lib/graphql/fragments/packets"

export const packetsLatestQueryDocument = graphql(
  /* GraphQL */ `
    query PacketsLatestQuery($limit: Int = 100) {
      v0_packets(limit: $limit, order_by: { source_time: desc_nulls_last }) {
        ...PacketListData
      }
    }
  `,
  [packetListDataFragment]
)
