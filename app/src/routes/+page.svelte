<script lang="ts">
import { Shine } from "svelte-ux"
</script>

<main class="mt-16 flex size-full min-size-full flex-col items-center justify-center">
  <Shine
    depth={1}
    surfaceScale={4}
    lightRadius={300}
    lightColor="#a0ecfd"
    specularExponent={120}
    specularConstant={0.75}
  >
    <h1
      class="~sm/md:~text-9xl/12xl font-black text-[9rem] leading-[9rem] brightness-60 cursor-default select-none"
    >
      zkGM
    </h1>
  </Shine>
</main>
