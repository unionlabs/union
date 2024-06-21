import { createQuery } from "@tanstack/svelte-query"
import { chainsQueryDocument } from "$lib/graphql/documents/chains"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const chainsQuery = () =>
  createQuery({
    queryKey: ["chains"],
    placeholderData: (previousData, _) => previousData,
    queryFn: async () => (await request(URLS.GRAPHQL, chainsQueryDocument, {})).v0_chains,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
