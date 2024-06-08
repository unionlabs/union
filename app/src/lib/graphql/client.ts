import {MutationCache, QueryClient} from "@tanstack/svelte-query"
import {createSyncStoragePersister} from "@tanstack/query-sync-storage-persister"
import {browser} from "$app/environment"

export function createQueryClient() {
  const queryClient: QueryClient = new QueryClient({
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

  return {queryClient, localStoragePersister}
}
