<script lang="ts">
import { onMount } from "svelte"

type Props = {
  progress: number
}

let { progress = 0 }: Props = $props()

let isVisible = $state(false)

let progressBar: HTMLElement

$effect(() => {
  const observer = new IntersectionObserver(
    ([entry]) => {
      if (entry.isIntersecting) {
        isVisible = true
        observer.unobserve(progressBar)
      }
    },
    {
      threshold: 0.1,
      rootMargin: "50px",
    },
  )

  if (progressBar) {
    observer.observe(progressBar)
  }

  return () => observer.disconnect()
})
</script>

<div
  class="w-full h-1 bg-zinc-800 overflow-hidden rounded-full"
  bind:this={progressBar}
>
  <div
    class="h-full bg-accent transition-all duration-1000"
    class:opacity-0={!isVisible}
    class:opacity-100={isVisible}
    style="width: {isVisible ? progress + '%' : '0%'}"
  >
  </div>
</div>
