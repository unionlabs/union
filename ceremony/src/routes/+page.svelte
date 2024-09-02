<script lang="ts">
  import { fetchStatus } from "$lib/data/api";
  import type { Status } from "$lib/data/api/types.ts";
  import { onMount, onDestroy } from "svelte";

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
