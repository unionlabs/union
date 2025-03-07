import {createQueryGraphql} from "$lib/utils/queries.ts";
import {Option, Schema} from "effect";
import {graphql} from "gql.tada";
import {chains} from "$lib/stores/chains.svelte.ts";
import {Channels} from "$lib/schema/channel.ts";
import {channels} from "$lib/stores/channels.svelte.ts";

export const channelsQuery = () =>
createQueryGraphql({
  schema: Schema.Struct({v1_ibc_union_channel_recommendations: Channels}),
  document: graphql(`
      query Ucs03Channels @cached(ttl: 60) {
          v1_ibc_union_channel_recommendations(where: {_and: [{version: {_eq: "ucs03-zkgm-0"}}, {destination_chain_id: {_neq: "11155111"}}, {destination_chain_id: {_neq: "17000"}}]}) {
              source_port_id
              source_chain_id
              source_channel_id
              source_connection_id
              destination_port_id
              destination_chain_id
              destination_channel_id
              destination_connection_id
          }
      }
  `),
  variables: {},
  refetchInterval: "60 seconds",
  writeData: data => {
    channels.data = data.pipe(Option.map(d => d.v1_ibc_union_channel_recommendations))
  },
  writeError: error => {
    chains.error = error
  }
})

