<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { block } from "$lib/stores/block.svelte"
import { chainsQuery } from "$lib/queries/chains.svelte"
import { chains } from "$lib/stores/chains.svelte"
import Sidebar from "$lib/components/layout/Sidebar/index.svelte"

let { children } = $props()

onMount(() => {
  const fiber = Effect.runFork(chainsQuery)
  return () =>
    Effect.runPromise(
      Effect.gen(function* () {
        yield* Fiber.interrupt(fiber)
        block.data = Option.none()
      })
    )
})
</script>

<div class="grid grid-cols-[auto_1fr] min-h-[100svh] w-full">
  <aside class="fixed top-0 left-0 bottom-0 w-64 dark:bg-zinc-800 shadow overflow-auto">
    <Sidebar/>
  </aside>
  
  <!-- Main content area: Has margin to clear fixed elements -->
  <main class="col-start-2 ml-64">
    <!-- Show any app-wide errors here -->
    {#if Option.isSome(chains.error)}
      <div class="bg-red-500">
        <h2 class="text-xl font-bold">Error updating chains</h2>
        <div>{chains.error.value}</div>
        <div>{chains.error.value.cause}</div>
      </div>
    {/if}
    {@render children()}
  </main>
</div>
