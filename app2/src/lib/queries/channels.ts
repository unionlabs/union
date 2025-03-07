import type {Environment} from "$lib/constants.ts";
import {createQueryGraphql} from "$lib/utils/queries.ts";
import {Option, Schema} from "effect";
import {Chains} from "$lib/schema/chain.ts";
import {graphql} from "gql.tada";
import {chains} from "$lib/stores/chains.svelte.ts";

export let recommendedUcs03ChannelsQuery = (environment: Environment) =>
createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_chains: Chains }),
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
  refetchInterval: "60 seconds",
  writeData: data => {
    chains.data = data.pipe(Option.map(d => d.v1_ibc_union_chains))
  },
  writeError: error => {
    chains.error = error
  }
})
