<script lang="ts">
  import '$/patch.ts'
  import '$styles/reset.css'
  import '$styles/tailwind.css'
  import { browser } from '$app/environment'
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query'
  import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools'

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        enabled: browser
      }
    }
  })
</script>

<QueryClientProvider client={queryClient}>
  <slot />
  <SvelteQueryDevtools initialIsOpen={false} />
</QueryClientProvider>

<style>
</style>
