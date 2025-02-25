<script lang="ts">
import "../app.css"
import { onMount } from "svelte"
import { Effect, Fiber, Option } from "effect"
import { block } from "$lib/stores/block.svelte"
import { chainsQuery } from "$lib/queries/chains.svelte"
import { chains } from "$lib/stores/chains.svelte"

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

{@render children()}

{#if Option.isSome(chains.error)}
  <div class="bg-red-500">
    <h2 class="text-xl font-bold">Error updating chains</h2>
    <div>{chains.error.value}</div>
    <div>{chains.error.value.cause}</div>
  </div>
{/if}
