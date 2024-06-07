import { createQuery } from "@tanstack/svelte-query"
import { chainsQueryDocument } from "$lib/graphql/documents/chains"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const chainsQuery = () =>
  createQuery({
    queryKey: ["chains"],
    queryFn: async () => request(URLS.GRAPHQL, chainsQueryDocument, {}),
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
