<script lang="ts">
  import '$/patch.ts'
  import '$styles/reset.css'
  import '$styles/tailwind.css'
  import { browser } from '$app/environment'
  import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools'
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query'

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { enabled: browser }
    }
  })
</script>

<QueryClientProvider client={queryClient}>
  <slot />
  <SvelteQueryDevtools initialIsOpen={false} />
</QueryClientProvider>
