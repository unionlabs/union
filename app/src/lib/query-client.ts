import { toast } from "svelte-sonner"
import { browser } from "$app/environment"
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from "@tanstack/svelte-query"

const SECOND = 1_000
const MINUTE = 60 * SECOND
const HOUR = 60 * MINUTE

export function createQueryClient() {
  const queryClient: QueryClient = new QueryClient({
    defaultOptions: {
      queries: {
        enabled: browser,
        gcTime: HOUR * 1, // 1 hour
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

  return {
    queryClient,
    QueryClientProvider
  }
}
