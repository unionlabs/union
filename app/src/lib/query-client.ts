import { toast } from "svelte-sonner"
import { browser } from "$app/environment"
import { MutationCache, QueryCache, QueryClient } from "@tanstack/svelte-query"
import { createSyncStoragePersister } from "@tanstack/query-sync-storage-persister"

export function createQueryClient() {
  const queryClient: QueryClient = new QueryClient({
    queryCache: new QueryCache({
      onError: (error, query) => {
        if (query.state.data !== undefined) {
          toast.error(`tanstack query error: ${error.message}`)
        }
      }
    }),
    defaultOptions: {
      queries: {
        enabled: browser,
        gcTime: 1000 * 60 * 60 * 24,
        refetchOnReconnect: () => !queryClient.isMutating()
      }
    },
    mutationCache: new MutationCache({
      onSettled: () => {
        if (queryClient.isMutating() === 1) {
          return queryClient.invalidateQueries()
        }
      }
    })
  })

  const localStoragePersister = createSyncStoragePersister({
    key: "SVELTE_QUERY",
    storage: typeof window !== "undefined" ? window.localStorage : undefined // Use local storage if in browser
  })

  return { queryClient, localStoragePersister }
}
