import { channels } from "$lib/stores/channels.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { Channels } from "@unionlabs/sdk/schema"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"

export const channelsQuery = () =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_channels: Channels }),
    document: graphql(`
      query Ucs03Channels @cached(ttl: 60) {
          v2_channels(args: { p_tags: ["canonical"]}) {
              destination_channel_id
              destination_client_id
              destination_connection_id
              destination_port_id
              fees {
                action
                fee
              }
              destination_universal_chain_id
              sort_order
              source_channel_id
              source_client_id
              source_connection_id
              source_port_id
              source_universal_chain_id
              version
              tags
          }
      }
  `),
    variables: {},
    refetchInterval: "1 hour",
    writeData: data => {
      channels.data = data.pipe(Option.map(d => d.v2_channels))
    },
    writeError: error => {
      channels.error = error
    },
  })
