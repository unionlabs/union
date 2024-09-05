<script lang="ts">
import { contribute, checkStatus } from "$lib/client/index.ts"
import Text from "$lib/components/typography/Text.svelte"
import Button from "$lib/components/Button.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"
import { checkContribution } from "$lib/supabase/index.ts"

let clientStore = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["client"],
    queryFn: () => checkStatus(),
    refetchInterval: 5_000,
    retry: false
  }))
)

let { error: clientError, isLoading: clientIsLoading, data: clientData } = $derived($clientStore)

let contributionStore = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["contribution"],
    queryFn: () => checkContribution(),
    refetchInterval: 5_000,
    retry: false
  }))
)

let {
  error: contributionError,
  isLoading: contributionIsLoading,
  data: contributionData
} = $derived($contributionStore)
</script>

<section class="w-full h-full flex items-center justify-center">
  <div class="max-w-7xl sm:px-6 lg:px-8">
    <!-- Client Status -->
    <div class="mb-4">
      {#if clientError}
        <Text>Client connected?</Text>
      {:else if clientIsLoading}
        <Spinner class="size-4 text-red-500"/>
      {:else if clientData}
        <Text>{clientData.status}</Text>
      {:else}
        <Text>Waiting for client...</Text>
      {/if}
    </div>

    <!-- Contribution Section -->
    <div>
      {#if contributionData}
        {#if contributionData.shouldContribute}
          <Button on:click={contribute}>Contribute</Button>
        {:else}
          <Text>Thanks for your contribution</Text>
        {/if}
      {/if}
    </div>
  </div>
</section>