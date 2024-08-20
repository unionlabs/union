import { createQuery } from "@tanstack/svelte-query"
import {
  statsQueryDocument,
  packetCountQueryDocument,
  transfersPerDayQueryDocument,
  transferCountQueryDocument
} from "$lib/graphql/queries/stats.ts"

import { request } from "graphql-request"
import { URLS } from "$lib/constants"

export const statsQuery = () =>
  createQuery({
    queryKey: ["stats"],
    queryFn: async () => (await request(URLS.GRAPHQL, statsQueryDocument, {})).v0_stats[0],
    enabled: true,
    refetchInterval: 5_000,
    refetchOnWindowFocus: false
  })

export const transfersPerDayQuery = (limit: number) =>
  createQuery({
    queryKey: ["transfer-per-day"],
    queryFn: async () =>
      (await request(URLS.GRAPHQL, transfersPerDayQueryDocument, { limit })).v0_daily_transfers,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })

export const packetCountQuery = () =>
  createQuery({
    queryKey: ["packet-count"],
    queryFn: async () =>
      (await request(URLS.GRAPHQL, packetCountQueryDocument, {})).v0_packets_aggregate,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })

export const transferCountQuery = () =>
  createQuery({
    queryKey: ["transfers-count"],
    queryFn: async () =>
      (await request(URLS.GRAPHQL, transferCountQueryDocument, {})).v0_transfers_aggregate,
    enabled: true,
    refetchInterval: 6_000,
    refetchOnWindowFocus: false
  })
