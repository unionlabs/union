<script lang="ts">
import { page } from "$app/stores"
</script>

<svelte:head>
  <title>{$page.status}</title>
</svelte:head>

<main class="size-full flex items-center mt-20 flex-col space-y-10 max-w-5xl">
  <p class="text-5xl font-black tracking-wider">{$page.status}</p>
  <p class="text-xl font-semibold">{$page.error?.message}</p>
  {#if $page.error && 'errorId' in $page.error}
    <div class="text-white/70 text-center">
      <p>error id</p>
      <p class="">{$page.error?.['errorId']}</p>
    </div>
  {/if}
</main>
