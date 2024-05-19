<script lang="ts">
import { Shine } from "svelte-ux"
import CardAnimatedBorder from "$lib/components/card-animated-border.svelte"
</script>

<main class="mt-16 flex size-full min-size-full flex-col items-center justify-center">
  <Shine depth={4} lightColor="#a0ecfd">
    <h1
      class="~sm/md:~text-9xl/11xl font-black leading-[9rem] brightness-75 cursor-default select-none text-center"
    >
      zkGM
    </h1>
  </Shine>
  <CardAnimatedBorder></CardAnimatedBorder>
</main>
