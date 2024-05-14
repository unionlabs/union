import { URLS } from "$lib/constants"
import { createQuery } from "@tanstack/svelte-query"

export function ibcConnectionsQuery() {
  return createQuery({
    queryKey: ["ibc-connections"],
    queryFn: async () => {
      const response = await fetch(`${URLS.UNION.REST}/ibc/core/connection/v1/connections`)
      const data = await response.json()
      return data
    },
    enabled: true
  })
}

export function ibcChannelsQuery() {
  return createQuery({
    queryKey: ["ibc-channels"],
    queryFn: async () => {
      const response = await fetch(`${URLS.UNION.REST}/ibc/core/channel/v1/channels`)
      const data = await response.json()
      return data
    },
    enabled: true
  })
}
