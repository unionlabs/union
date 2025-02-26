<script lang="ts">
import { transferListLatestQuery } from "$lib/queries/transfer-list.svelte"
import { Effect, Fiber, Option } from "effect"
import { onMount } from "svelte"
import { transferList } from "$lib/stores/transfers.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"

onMount(() => {
  const fiber = Effect.runFork(transferListLatestQuery)
  return () => Effect.runPromise(Fiber.interrupt(fiber))
})
</script>


<Sections>
  <Card class="overflow-auto">
    {#if Option.isSome(transferList.data)}
      <pre>{JSON.stringify(transferList.data.value, null,2)}</pre>
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {:else}
      Loading...
      {#if Option.isSome(transferList.error)}
        <ErrorComponent error={transferList.error.value}/>
      {/if}
    {/if}
  </Card>
</Sections>
