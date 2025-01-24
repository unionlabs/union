import { writable, type Writable } from "svelte/store"

type HighlightItem = { kind: "address"; address: string } | { kind: "token"; denom: string } | null
export const highlightItem: Writable<HighlightItem> = writable(null)
