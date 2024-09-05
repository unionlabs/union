<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import { checkContribution, checkPosition } from "$lib/api"
import { createQuery } from "@tanstack/svelte-query"
import Spinner from "$lib/components/Spinner.svelte"
import Link from "$lib/components/typography/Link.svelte"
import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"

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

{#if user.session}
  <div class="max-w-7xl px-6 lg:px-8 py-24 overflow-y-scroll">
    <H1>{user.session.user.email}</H1>
    <div class="h-4">
      <Text>
        {#if data && !isLoading && !contributionIsLoading}
          {#if data.position === 1 && contributionData?.shouldContribute}
            It's Your turn! <Link href="/app/client" class="font-bold">Click here</Link>
          {:else if data.position > 1}
            Your position in the q: {data.position}
          {:else}
            Thanks for your contribution
          {/if}
        {:else }
          <Spinner class="size-4"/>
        {/if}
      </Text>
    </div>
  </div>
{/if}