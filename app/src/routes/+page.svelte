<script lang="ts">
import { onMount } from "svelte"
import { cn } from "$lib/utilities/shadcn.ts"

onMount(() => {
  console.info()
})
</script>

<svelte:head></svelte:head>

<main
  class={cn(
    'mt-12 flex size-full min-w-full min-h-full flex-col items-center justify-center space-y-6 dark:text-white',
  )}
>
  <w3m-button enable="true" />
  <h1 class="text-9xl font-black">zkGM</h1>
</main>
