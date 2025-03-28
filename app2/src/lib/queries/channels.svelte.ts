import { createQueryGraphql } from "$lib/utils/queries.ts"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { chains } from "$lib/stores/chains.svelte.ts"
import { Channels } from "$lib/schema/channel.ts"
import { channels } from "$lib/stores/channels.svelte.ts"

export const channelsQuery = () =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_channels: Channels }),
    document: graphql(`
      query Ucs03Channels @cached(ttl: 60) {
          v2_channels(args: {p_recommended: true}) {
              destination_channel_id
              destination_client_id
              destination_connection_id
              destination_port_id
              destination_universal_chain_id
              source_channel_id
              source_client_id
              source_connection_id
              source_port_id
              source_universal_chain_id
          }
      }
  `),
    variables: {},
    refetchInterval: "60 seconds",
    writeData: data => {
      channels.data = data.pipe(Option.map(d => d.v2_channels))
    },
    writeError: error => {
      chains.error = error
    }
  })
