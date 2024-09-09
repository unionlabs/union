<script lang="ts">
  import { user } from "$lib/stores/user.svelte.ts"
  import Text from "$lib/components/typography/Text.svelte"
  import { createQuery } from "@tanstack/svelte-query"
  import Spinner from "$lib/components/Spinner.svelte"
  import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"
  import H2 from "$lib/components/typography/H2.svelte"
  import { checkContributionStatus, getUserQueueInfo } from "$lib/supabase"
  import Button from "$lib/components/Button.svelte"
  import { checkStatus, start } from "$lib/client";
  import Install from "$lib/components/Install.svelte";

  let clientQuery = createQuery(
    reactiveQueryArgs(() => ({
      queryKey: ["client"],
      queryFn: () => checkStatus(),
      refetchInterval: 5_000,
      retry: false
    }))
  )

  let contributionQuery = createQuery(
    reactiveQueryArgs(() => ({
      queryKey: ["contribution"],
      queryFn: () => checkContributionStatus(),
      refetchInterval: 5_000,
      retry: false
    }))
  )

  let queueQuery = createQuery(
    reactiveQueryArgs(() => ({
      queryKey: ["userPosition"],
      queryFn: () => getUserQueueInfo(),
      refetchInterval: 5_000,
      retry: 2,
      retryDelay: 1000
    }))
  )

  let {data: queue, isLoading: queueLoading, error: queueError} = $derived($queueQuery)
  let {data: contribute, isLoading: contributeLoading, error: contributeError} = $derived($contributionQuery)
  let {data: client, isLoading: clientLoading, error: clientError} = $derived($clientQuery)

</script>


<div class="p-8  bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center flex-col">

  <Text class="uppercase">USER: <span class="text-union-accent-500">{user?.session?.user.email}</span></Text>

<!--  First we check if the user is in the queue, if so we show the queue position.

  If not they might be the current contributor or already contributed or we are verifying their contribution.

  If they are able to contribute they need to have the client installed.-->

  {#if queueLoading}
    <Spinner class="size-4 text-union-accent-500"/>
  {:else if queue.inQueue}

    <H2>You are in queue</H2>
    <Text>Position: {queue.position}/{queue.count + 1}</Text>

  {:else if contribute?.canContribute && contribute?.shouldContribute}

    {#if clientError}
      <Text>Client connected?</Text>
      <Install />
    {:else if clientLoading}
      <Spinner class="size-4 text-red-500"/>
    {:else if client}
      <Text>{client.status}</Text>
      <H2>You can contribute</H2>
      <Button onclick={start}>Contribute</Button>
    {:else}
      <Text>Waiting for client...</Text>
    {/if}

  {:else if contribute?.isVerifying}

    <H2>Your contribution is being verified</H2>

  {:else}

    <H2>Not eligible for contribution at this time</H2>

  {/if}

</div>
