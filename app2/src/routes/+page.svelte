<script lang="ts">
import { onMount } from "svelte"
import { createQuery, type FetchDecodeError } from "$lib/utils/queries"
import { Effect, Fiber, Option } from "effect"
import { Block } from "$lib/schemas/block"
import { block } from "$lib/stores/block.svelte"

const program = createQuery({
  url: "https://rpc.testnet-9.union.build/block",
  schema: Block,
  refetchInterval: "4 seconds",
  writeData: data => {
    block.data = data
  },
  writeError: error => {
    block.error = error
  }
})

onMount(() => {
  const fiber = Effect.runFork(program)
  return () =>
    Effect.runPromise(
      Effect.gen(function* () {
        yield* Fiber.interrupt(fiber)
        block.data = Option.none()
      })
    )
})
</script>

{#if Option.isSome(block.data)}
  <div class="font-mono">
    {block.data.value.result.block_id.hash}
  </div>
  {#if Option.isSome(block.error)}
    There was an error refetching this data. the data you see is stale
    <pre class="font-mono bg-red-500">
      {JSON.stringify(block.error.value, null, 2)}
    </pre>
  {/if}
{:else}
  Loading...
  {#if Option.isSome(block.error)}
    There was an error fetching this data. We will retry automatically. Error:
    <pre class="font-mono bg-red-500">
      {JSON.stringify(block.error.value, null, 2)}
    </pre>
  {/if}
{/if}
