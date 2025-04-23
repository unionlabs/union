<script lang="ts">
import { Effect } from "effect"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { cn } from "$lib/utils"

const HEALTH_ENDPOINTS = {
  production: "https://graphql.union.build/api/rest/health",
  staging: "https://staging.graphql.union.build/api/rest/health",
  development: "https://development.graphql.union.build/api/rest/health"
} as const

let healthData: Array<{
  environment: string
  status: string
  lastUpdate: string
  color: string
  error?: string
}> = []

const fetchHealth = (url: string) =>
  Effect.tryPromise({
    try: async () => {
      const response = await fetch(url)
      const data = await response.json()
      return {
        status: data.v2_health_check[0].status,
        lastUpdate: data.v2_health_check[0].last_update,
        color: data.v2_health_check[0].environment
      }
    },
    catch: error => ({
      status: "ERROR",
      lastUpdate: new Date().toISOString(),
      color: "gray",
      error: error instanceof Error ? error.message : "Unknown error"
    })
  })

async function fetchAllHealth() {
  const data = await Effect.all(
    Object.entries(HEALTH_ENDPOINTS).map(([env, url]) =>
      fetchHealth(url).pipe(
        Effect.map(data => ({
          environment: env,
          ...data
        }))
      )
    )
  ).pipe(Effect.runPromise)

  healthData = data
}

// Initial fetch
fetchAllHealth()

// Refresh every 30 seconds
setInterval(fetchAllHealth, 30000)
</script>

<Sections>
    <Card class="overflow-auto" divided>
        <div class="p-3 text-sm font-medium text-zinc-400">Health Status</div>
        <div class="space-y-1">
            {#each healthData as item}
                <a 
                    href={HEALTH_ENDPOINTS[item.environment as keyof typeof HEALTH_ENDPOINTS]} 
                    target="_blank"
                    rel="noopener noreferrer"
                    class={cn(
                        "flex justify-between gap-8 px-4 py-2 h-12 items-center",
                        "hover:bg-zinc-50 dark:hover:bg-zinc-900 transition-colors duration-75",
                        "cursor-pointer"
                    )}>
                    <div>
                        <h2 class="text-sm font-medium capitalize">{item.environment}</h2>
                        <p class="text-xs text-zinc-400">Last updated: {new Date(item.lastUpdate).toLocaleString()}</p>
                    </div>
                    <div class="flex items-center gap-2">
                        <span class={cn(
                            "px-2 py-0.5 text-xs font-medium rounded-sm",
                            item.status === 'OK' ? "bg-emerald-500/20 text-emerald-500" : "bg-red-500/20 text-red-500"
                        )}>
                            {item.status}
                        </span>
                        <span class="px-2 py-0.5 text-xs font-medium bg-zinc-500/20 text-zinc-500 rounded-sm">
                            {item.color}
                        </span>
                    </div>
                    {#if item.error}
                        <p class="text-xs text-red-500">Error: {item.error}</p>
                    {/if}
                </a>
            {/each}
        </div>
    </Card>
</Sections>
