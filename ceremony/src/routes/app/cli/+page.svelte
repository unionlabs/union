<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {fetchStatus, type Status} from "$lib/api/index.ts";
  import Text from "$lib/components/typography/Text.svelte";
  import H1 from "$lib/components/typography/H1.svelte";

  let status: Status | undefined = $state(undefined);
  let intervalId: number | NodeJS.Timeout

  onMount(async () => {
    status = await fetchStatus();

    console.log(status)
    intervalId = setInterval(async () => {
      status = await fetchStatus();
    }, 30000);
  });

  onDestroy(() => {
    clearInterval(intervalId);
  });
</script>

<section class="w-full h-full flex items-center justify-center">
  <div class="max-w-7xl sm:px-6 lg:px-8">
    <H1>⨌⨀_⨀⨌</H1>
    <Text>{status ? status : "Not connected"}</Text>
  </div>
</section>
