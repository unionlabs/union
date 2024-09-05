<script lang="ts">
import { contribute, checkStatus } from "$lib/api/index.ts"
import Text from "$lib/components/typography/Text.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { reactiveQueryArgs, user } from "$lib/stores/user.svelte.ts"
import Button from "$lib/components/Button.svelte"
import { createQuery } from "@tanstack/svelte-query"
import Spinner from "$lib/components/Spinner.svelte"

let clientStore = createQuery(
  reactiveQueryArgs(() => ({
    queryKey: ["client"],
    queryFn: () => checkStatus(),
    refetchInterval: 5_000,
    retry: false
  }))
)

let { error, isLoading, data, status } = $derived($clientStore)

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
    {#if error}
      <Text>Client connected?</Text>
      {:else if isLoading}
      <Spinner class="size-4 text-red-500"/>
      {:else if data}
      <Text>{data.status}</Text>
      <Button onclick={start}>Contribute</Button>
    {/if}
  </div>
</section>