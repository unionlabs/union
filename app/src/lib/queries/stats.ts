import { createQuery } from "@tanstack/svelte-query";
import { transferCountQueryDocument, transfersPerDayQueryDocument } from "$lib/graphql/documents/stats.ts";

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const transferPerDayQuery = () =>
  createQuery({
      queryKey: ["transfers"],
      queryFn: async () => (await request(URLS.GRAPHQL, transfersPerDayQueryDocument, { limit: 24 })).v0_daily_transfers
      ,
      enabled: true,
      refetchInterval: 6_000,
      refetchOnWindowFocus: false
  })
