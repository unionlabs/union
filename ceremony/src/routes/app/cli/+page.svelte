<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {fetchStatus, type Status} from "$lib/api/index.ts";

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
    <p>⨌⨀_⨀⨌</p>
    <p>{status ? status : "Not connected"}</p>
  </div>
</section>
