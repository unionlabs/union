<script lang="ts">
import { contribute, checkStatus, checkContribution } from "$lib/api/index.ts"
import Text from "$lib/components/typography/Text.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"
import Button from "$lib/components/Button.svelte"
import { createQuery } from "@tanstack/svelte-query"
import { reactiveQueryArgs } from "$lib/utils/utils.svelte.ts"
import Spinner from "$lib/components/Spinner.svelte"

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

const start = async () => {
  if (!user?.session) {
    console.error("User not logged in")
    return
  }

  const { data, error } = await supabase
    .from("queue")
    .select("payload_id")
    .filter("id", "eq", user.session.user.id)
    .single()

  if (error) {
    console.error("Error fetching payload_id:", error)
    return
  }

  if (!data) {
    console.log("No data found for the given user ID")
    return
  }

  await contribute({ payloadId: data.payload_id })
}
</script>

<section class="w-full h-full flex items-center justify-center">
  <div class="max-w-7xl sm:px-6 lg:px-8">
    {#if contributionData?.shouldContribute}
      {#if clientError}
        <Text>Client connected?</Text>
      {:else if clientIsLoading}
        <Spinner class="size-4 text-red-500"/>
      {:else if clientData}
        <Text>{clientData.status}</Text>
        {#if contributionData?.shouldContribute}
          <Button onclick={start}>Contribute</Button>
        {:else}
          <Text>Thanks for your contribution</Text>
        {/if}
      {/if}
    {:else}
      <Text>Thanks for your contribution</Text>
    {/if}
  </div>
</section>