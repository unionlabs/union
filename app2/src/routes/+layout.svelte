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

<div class="grid grid-cols-[auto_1fr] grid-rows-[auto_1fr] min-h-[100svh] w-full">
  <header class="col-span-2 fixed top-0 left-0 right-0 h-16 z-10 dark:bg-zinc-800 shadow">
    <div class="px-4 h-full flex items-center">
      Header Content
    </div>
  </header>
  
  <aside class="fixed top-16 left-0 bottom-0 w-64 dark:bg-zinc-800 shadow overflow-auto">
    <Sidebar/>
  </aside>
  
  <!-- Main content area: Has margin to clear fixed elements -->
  <main class="col-start-2 mt-16 ml-64">
    <!-- Main scrollable content -->
    {@render children()}
  </main>
</div>

<!--
<Sidebar/>
{#if Option.isSome(chains.error)}
  <div class="bg-red-500">
    <h2 class="text-xl font-bold">Error updating chains</h2>
    <div>{chains.error.value}</div>
    <div>{chains.error.value.cause}</div>
  </div>
{/if}
!-->
