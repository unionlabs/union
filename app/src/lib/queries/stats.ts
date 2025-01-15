import { createQuery } from "@tanstack/svelte-query"
import { statsQueryDocument, transfersPerDayQueryDocument } from "$lib/graphql/queries/stats.ts"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"
import type { QueryObserverResult } from "@tanstack/query-core"
import type { Readable } from "svelte/store"

interface DailyTransfer {
  count: number
  day: string
}

interface Statistic {
  name: string
  value: number
}

export const statsQuery = (): Readable<QueryObserverResult<Array<Statistic>, Error>> =>
  createQuery({
    queryKey: ["stats"],
    queryFn: async () => {
      const response = await request(URLS().GRAPHQL, statsQueryDocument, {})
      return response.v1_ibc_union_statistics
    },
    enabled: true,
    refetchInterval: 5_000,
    refetchOnWindowFocus: false
  })

export const transfersPerDayQuery = (
  limit: number
): Readable<QueryObserverResult<Array<DailyTransfer>, Error>> =>
  createQuery({
    queryKey: ["transfer-per-day"],
    queryFn: async () => {
      const response = await request(URLS().GRAPHQL, transfersPerDayQueryDocument, { limit })
      return response.v1_ibc_union_daily_transfers
    },
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
