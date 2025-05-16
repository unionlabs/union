<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import NewUser from "$lib/dashboard/components/NewUser.svelte"
import { getLastGlobalSync } from "$lib/dashboard/services/cache"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { Effect } from "effect"
import type { Snippet } from "svelte"
import { onDestroy, onMount } from "svelte"

interface Props {
  children: Snippet
}

let { children }: Props = $props()

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

$effect(() => {
  if (Option.isSome(dashboard.missions)) {
    console.log(dashboard.missions.value.completed)
  }
})
</script>

<Sections>
  {#if lastSyncLabel}
    <p class="text-xs text-zinc-400 -my-4 px-2 text-end">{lastSyncLabel}</p>
  {/if}
  <NewUser />
  {@render children()}
</Sections>
