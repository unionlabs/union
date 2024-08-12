import { URLS } from "$lib/constants"
import { request } from "graphql-request"
import { createQuery } from "@tanstack/svelte-query"
import { chainsQueryDocument } from "$lib/graphql/queries/chains"

export const chainsQuery = () =>
  createQuery({
    queryKey: ["chains"],
    placeholderData: (previousData, _) => previousData,
    queryFn: async () => {
      const { data } = await request(URLS.GRAPHQL, chainsQueryDocument, {
        includeRpcs: true,
        includeContracts: true,
        includeEndpoints: true
      })
      return data
    },
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
