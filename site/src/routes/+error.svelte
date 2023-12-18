<script lang="ts">
  import { page } from '$app/stores'
</script>

<main class="flex items-center justify-center flex-1 flex-col">
  <h1 class="text-3xl font-bold">Error Loading Page</h1>
  <p class="italic text-xl">Sorry about that.</p>
  <a class="font-mono text-accent text-xl" href="/">Back to home -&gt;</a>

  {#if $page?.error}
    <div class="mt-16">
      <h2 class="font-bold">Details:</h2>
      {#if $page?.status}
        <p>Page status: {$page?.status}</p>
      {/if}
      {#if $page?.error?.message}
        <p>Error message: {$page?.error?.message}</p>
      {/if}
    </div>
  {/if}
</main>
