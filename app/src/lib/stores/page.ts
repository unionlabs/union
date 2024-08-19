import { page } from "$app/stores"
import { decodeTimestampSearchParam } from "$lib/timestamps"
import { derived, type Readable } from "svelte/store"

export const timestamp: Readable<string | null> = derived(page, $page => {
  const urlTimestamp = $page.url.searchParams.get("timestamp")
  if (!urlTimestamp) return null
  return decodeTimestampSearchParam(urlTimestamp)
})
