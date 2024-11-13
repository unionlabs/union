import { createQuery } from "@tanstack/svelte-query"
import { chainsQueryDocument } from "$lib/graphql/queries/chains"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"
import { reactiveQueryArgs } from '$lib/utilities/utilities.svelte'

export const chainsQuery = () =>
  createQuery(
    reactiveQueryArgs(() => ({
      queryKey: ["chains"],
// placeho lderData: _ => {},
      // placeholderData: (previousData, _) => previousData,
      queryFn: async () => (await request(URLS().GRAPHQL, chainsQueryDocument, {})).v1_chains,
      enabled: true,
      refetchInterval: 6_000,
      refetchOnWindowFocus: false
    }))
  )
