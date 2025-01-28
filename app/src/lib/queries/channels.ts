import { createQuery } from "@tanstack/svelte-query"
import { recommendedUcs03ChannelsQueryDocument } from "$lib/graphql/queries/channels"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const recommendedUcs03ChannelsQuery = () =>
  createQuery({
    queryKey: ["recommended-ucs03-channels"],
    placeholderData: (previousData, _) => previousData,
    queryFn: async () =>
      (await request(URLS().GRAPHQL, recommendedUcs03ChannelsQueryDocument, {}))
        .v1_ibc_union_channel_recommendations,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
