<script lang="ts">
import { getLastGlobalSync } from "$lib/dashboard/services/cache"
import { Effect } from "effect"
import { Option } from "effect"
import { onDestroy, onMount } from "svelte"

let lastSyncLabel: string | null = $state(null)

const updateLastSync = () => {
  Effect.runPromise(getLastGlobalSync()).then(meta => {
    lastSyncLabel = Option.isSome(meta) ? meta.value.label : null
  }).catch(error => {
    console.error("Failed to get last sync time:", error)
    lastSyncLabel = null
  })
}

onMount(() => {
  updateLastSync()
  const interval = setInterval(updateLastSync, 60_000)

  onDestroy(() => clearInterval(interval))
})
</script>

{#if lastSyncLabel}
  <p class="text-xs text-zinc-400 -my-4 px-2 text-end">{lastSyncLabel}</p>
{/if}
