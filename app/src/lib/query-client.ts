import { browser } from "$app/environment"
import { MutationCache, QueryClient, QueryCache } from "@tanstack/svelte-query"
import { createSyncStoragePersister } from "@tanstack/query-sync-storage-persister"
import { toast } from "svelte-sonner"
import { page } from "$app/stores"
import { get } from "svelte/store"

export function createQueryClient() {
  const queryClient: QueryClient = new QueryClient({
    queryCache: new QueryCache({
      /**
       * https://tkdodo.eu/blog/react-query-error-handling#putting-it-all-together
       * shows a toast message when an error occurs in development mode
       */
      onError: (error, query) => {
        if (import.meta.env.MODE !== "development") return
        if (!get(page).url.host.includes("localhost")) return

        if (query.state.data !== undefined) {
          toast.error(`Tanstack Error: ${error.message}`)
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
