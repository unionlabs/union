<script lang="ts">
import { user } from "$lib/stores/user.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import { createQuery } from "@tanstack/svelte-query"
import Spinner from "$lib/components/Spinner.svelte"
import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"
import H2 from "$lib/components/typography/H2.svelte"
import { checkContributionStatus, getUserQueueInfo } from "$lib/supabase"
import Button from "$lib/components/Button.svelte"
import { goto } from "$app/navigation"

let userQueueStats = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["userPosition"],
    queryFn: () => getUserQueueInfo(),
    refetchInterval: 5_000,
    retry: 2,
    retryDelay: 1000
  }))
)

let { isLoading, data: userQueue, error } = $derived($userQueueStats)

let contributionStore = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["contribution"],
    queryFn: () => checkContributionStatus(),
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


<div class="p-8  bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center flex-col">

  <Text class="uppercase">USER: <span class="text-union-accent-500">{user?.session?.user.email}</span></Text>

  {#if isLoading}
    <Spinner class="size-4 text-union-accent-500"/>
  {:else if userQueue.inQueue}
    <H2>You are in queue</H2>
    <Text>Position: {userQueue.position}/{userQueue.count + 1}</Text>
  {:else if contributionData?.canContribute && contributionData?.shouldContribute}
    <H2>You can contribute</H2>
    <Button onclick={() => goto("/app/client")}>Contribute</Button>
  {:else if contributionData?.isVerifying}
    <H2>Your contribution is being verified</H2>
  {:else}
    <H2>Not eligible for contribution at this time</H2>
  {/if}
</div>
