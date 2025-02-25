<script lang="ts">
import { transferListQuery } from "$lib/queries/transfer-list.svelte"
import { Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"

onMount(() => {
  const fiber = Effect.runFork(transferListQuery)
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})
</script>


{#if Option.isSome(transferList.data)}
  <pre>{JSON.stringify(transferList.data.value, null,2)}</pre>
  {#if Option.isSome(transferList.error)}
    <pre>{JSON.stringify(transferList.error.value, null,2)}</pre>
  {/if}
{:else}
  Loading...
  {#if Option.isSome(transferList.error)}
    <pre>{JSON.stringify(transferList.error.value, null,2)}</pre>
  {/if}
{/if}



