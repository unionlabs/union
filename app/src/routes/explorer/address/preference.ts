import { persistedWritable } from "$lib/stores/persisted.ts"

/**
 * User dashboard address transfers preferences
 * with default values: rowsPerPage: 10, refreshInterval: 6_000
 */

export const addressTransfersPreference = persistedWritable("explorer:transfers:address", {
  rowsPerPage: Number.parseInt(import.meta.env.VITE_EXPLORER_ADDRESS_TRANSFERS_ROWS_PER_PAGE ?? 10),
  refreshInterval: Number.parseInt(
    import.meta.env.VITE_EXPLORER_ADDRESS_TRANSFERS_REFETCH_INTERVAL ?? 6_000 // 6 seconds
  )
})
