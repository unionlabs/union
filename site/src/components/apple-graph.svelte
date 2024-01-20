<script lang="ts">
  import { onMount } from 'svelte'

  // The percentage width of the bar. number from 0-100.
  export let width: number
  export let primary = false
  export let label: string

  let observer: null | IntersectionObserver = null

  let isVisible = false

  onMount(() => {
    // The HTML Element to observe.
    const element = document.querySelector('div[data-graph="apple"]') as HTMLElement
    observer = new IntersectionObserver(entries => {
      isVisible = isVisible || entries[0].isIntersecting
    })

    observer.observe(element)

    return () => {
      if (observer) {
        observer.disconnect()
        observer = null
      }
    }
  })
</script>

<div
  data-graph="apple"
  class="bg-transparent flex meter h-2 w-full"
>
  <span
    class="flex"
    style={`width:${width}%;`}
  >
    <span
      class={`rounded-full flex ${primary ? 'bg-white' : 'bg-gray-500'}` +
        (isVisible ? ' progress' : '')}
    >
    </span>
  </span>
</div>
<span>
  {label}
</span>

<style>
  .progress {
    animation: progressBar 1.5s ease-in-out;
    animation-fill-mode: both;
  }

  @keyframes progressBar {
    0% {
      width: 0;
    }

    100% {
      width: 100%;
    }
  }
</style>
