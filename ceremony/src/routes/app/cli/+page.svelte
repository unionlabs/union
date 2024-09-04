<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {contribute, fetchStatus} from "$lib/api/index.ts";
  import Text from "$lib/components/typography/Text.svelte";
  import H1 from "$lib/components/typography/H1.svelte";
  import type {Status} from "$lib/api/types.ts";

  import {supabase} from "$lib/supabase/client.ts";
  import Button from "$lib/components/Button.svelte";
  import {user} from "$lib/stores/user.svelte.ts";

  let status: Status | undefined = $state(undefined);
  let intervalId: number | NodeJS.Timeout

  onMount(async () => {
    status = await fetchStatus();

    console.log(status)
    intervalId = setInterval(async () => {
      status = await fetchStatus();
    }, 1000 * 5);
  });

  onDestroy(() => {
    clearInterval(intervalId);
  });

  const start = async () => {
    const { data, error } = await supabase
      .from("queue")
      .select("payload_id")
      .filter("id", "eq", user?.session?.user.id);

    if (error) {
      console.error("Error fetching payload_id:", error);
    } else if (data && data.length > 0) {
      const [{ payload_id }] = data;

      console.log({
        contributor_id: user?.session?.user.id,
        jwt: user?.session?.access_token,
        payload_id
      })
      const here = await contribute({
        contributorId: user?.session?.user.id,
        jwt: user?.session?.access_token,
        payloadId: payload_id
      });
      console.log(here)
    } else {
      console.log("No data found for the given user ID");
    }
  }

  $effect(() => {
    console.log(status)
  })
</script>

<section class="w-full h-full flex items-center justify-center">
  <div class="max-w-7xl sm:px-6 lg:px-8">
    <H1>⨌⨀_⨀⨌</H1>
    <Text>{status?.status ? status.status : status ? status : 'Not connected'}</Text>
    <Button onclick={start}>Contribute</Button>
  </div>
</section>
