<script lang="ts">
import { Shine } from "svelte-ux"
import { cn } from "$lib/utilities/shadcn.ts"
</script>

<svelte:head></svelte:head>

<main
  class={cn(
    'mt-12 flex size-full min-w-full min-h-full flex-col items-center justify-center space-y-6 dark:text-white',
  )}
>
  <Shine depth={3} lightColor="#a0ecfd">
    <h1 class="~sm/md:~text-8xl/9xl font-black text-[9rem] leading-[9rem] grayscale-[50] brightness-50 cursor-default">zkGM</h1>
  </Shine>
</main>
