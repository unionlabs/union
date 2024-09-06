<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import { createQuery } from "@tanstack/svelte-query"
import Spinner from "$lib/components/Spinner.svelte"
import Link from "$lib/components/typography/Link.svelte"
import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"
import H2 from "$lib/components/typography/H2.svelte"
import { checkContribution, checkQueue } from "$lib/supabase/index.ts"


let position = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["queue"],
    queryFn: () => checkQueue(),
    refetchInterval: 5_000,
    retry: 2,
    retryDelay: 1000
  }))
)

let { isLoading: queueIsLoading, data: queueData } = $derived($position)

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


<div class="resize w-full h-full px-6 lg:px-8 py-24">

  <Text class="uppercase">USER: <span class="text-union-accent-500">{user?.session?.user.email}</span></Text>

  {#if queueData && contributionData && !queueIsLoading && !contributionIsLoading}
    {#if queueData.position > 1}
      <H2>You are <span class="text-union-accent-500">{queueData.position}th</span> in queue</H2>
      <Text>Queue length {queueData?.total}</Text>
    {:else if queueData.position === 1 && contributionData?.shouldContribute}
      <H2>It's Your turn!</H2>
      <Link href="/app/client" class="font-bold">Click here</Link>
    {:else if contributionData?.status === 'contributed'}
      <H2>Contributed</H2>
    {/if}
  {:else}
    <Spinner class="size-6 text-union-accent-500"/>
  {/if}



</div>
