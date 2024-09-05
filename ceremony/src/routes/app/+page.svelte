<script lang="ts">
import { reactiveQueryArgs, user } from "$lib/stores/user.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import { checkPosition } from "$lib/api"
import { createQuery } from "@tanstack/svelte-query"
import Spinner from "$lib/components/Spinner.svelte"

let position = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["position"],
    queryFn: () => checkPosition(),
    refetchInterval: 5_000,
    retry: 2,
    retryDelay: 1000
  }))
)

let { error, isLoading, isRefetching, data } = $derived($position)
</script>

{#if user.session}
  <div class="max-w-7xl px-6 lg:px-8 py-24 overflow-y-scroll">
    <H1>{user.session.user.email}</H1>
    <div class="h-4">
      <Text>
        {#if data && !isLoading}
          {#if data.position === 1}
            Your turn!
            {:else}
            Your turn in
          {/if}
          {:else }
          <Spinner class="size-4"/>
        {/if}
      </Text>
    </div>
  </div>
{/if}