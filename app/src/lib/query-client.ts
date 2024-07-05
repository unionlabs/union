import { toast } from "svelte-sonner"
import { browser } from "$app/environment"
import { PersistQueryClientProvider } from "@tanstack/svelte-query-persist-client"
import { createSyncStoragePersister } from "@tanstack/query-sync-storage-persister"
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from "@tanstack/svelte-query"

export function createQueryClient() {
  const queryClient: QueryClient = new QueryClient({
    defaultOptions: {
      queries: {
        enabled: browser,
        gcTime: 1000 * 60 * 60 * 24,
        refetchOnReconnect: () => !queryClient.isMutating()
      }
    },
    /**
     * https://tkdodo.eu/blog/react-query-error-handling#putting-it-all-together
     * note: only runs in development mode. Production unaffected.
     */
    queryCache: new QueryCache({
      onError: (error, query) => {
        if (import.meta.env.MODE !== "development") return
        if (query.state.data !== undefined) {
          toast.error(`Tanstack Query Error: ${error.message}`)
        }
      }
    }),
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

  return {
    queryClient,
    QueryClientProvider,
    localStoragePersister,
    PersistQueryClientProvider
  }
}
