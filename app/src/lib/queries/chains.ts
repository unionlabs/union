import { URLS } from "$lib/constants"
import { request } from "graphql-request"
import { createQuery } from "@tanstack/svelte-query"
import { chainsQueryDocument } from "$lib/graphql/queries/chains"

export const chainsQueryKeys = {
  all: ["chains"] as const,
  list: (filters: string) => [...chainsQueryKeys.all, { filters }] as const
}

export const chainsQuery = () =>
  createQuery({
    queryKey: chainsQueryKeys.all,
    placeholderData: (previousData, _) => previousData,
    queryFn: async () => (await request(URLS().GRAPHQL, chainsQueryDocument, {})).v1_chains,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
