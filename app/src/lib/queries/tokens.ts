import { createQuery } from "@tanstack/svelte-query"
import { tokensQueryDocument } from "$lib/graphql/queries/tokens"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const tokensQuery = () =>
  createQuery({
    queryKey: ["tokens"],
    placeholderData: (previousData, _) => previousData,
    queryFn: async () =>
      (await request(URLS().GRAPHQL, tokensQueryDocument, {})).v1_ibc_union_tokens,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
