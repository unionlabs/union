<script lang="ts">
import clsx from "clsx"
import { onMount } from "svelte"

let [progress, visible] = [0, false]
onMount(() => {
  visible = true
  function next() {
    progress += 0.1
    const remaining = 1 - progress
    if (remaining > 0.15) setTimeout(next, 500 / remaining)
  }
  setTimeout(next, 250)
})
</script>

{#if visible}
  <div class="absolute w-full h-1 z-[999] left-0 top-0">
    <div
      style="width: {progress * 100}%"
      class={clsx(['absolute h-full bg-cyan-300 transition-[width] duration-[0.4s] left-0 top-0'])}
    />
  </div>
{/if}

{#if progress >= 0.4}
  <div
    class="fixed w-full h-full bg-[rgba(234,248,255,0.04)] pointer-events-none z-[998] animate-[fade_0.4s]"
  />
{/if}

<style lang="postcss">
  @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
