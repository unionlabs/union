<script lang="ts">
import type { Snippet } from "svelte"
import { fly } from "svelte/transition"

interface Props {
  class?: string
  currentSlide?: number
  totalSlides: number
  onSlideChange?: (slideIndex: number) => void
  children: Snippet<[number]>
}

let {
  class: className = "",
  currentSlide = $bindable(0),
  totalSlides,
  onSlideChange,
  children,
}: Props = $props()

let previousSlide = $state(0)
let direction = $derived(currentSlide > previousSlide ? 1 : -1)

function goToSlide(slideIndex: number) {
  if (slideIndex >= 0 && slideIndex < totalSlides) {
    previousSlide = currentSlide
    currentSlide = slideIndex
    onSlideChange?.(slideIndex)
  }
}

function goToNextSlide() {
  if (currentSlide < totalSlides - 1) {
    goToSlide(currentSlide + 1)
  }
}

function goToPreviousSlide() {
  if (currentSlide > 0) {
    goToSlide(currentSlide - 1)
  }
}

export { goToNextSlide, goToPreviousSlide, goToSlide }
</script>

<div class="relative overflow-hidden {className}">
  <div class="grid w-full grow overflow-hidden h-full">
    {#each Array(totalSlides) as _, slideIndex}
      {#if slideIndex === currentSlide}
        <div
          class="flex grow col-start-1 col-end-2 row-start-1 row-end-2"
          out:fly={{ x: direction * -382, duration: 300 }}
          in:fly={{ x: direction * 382, duration: 300 }}
        >
          {@render children(slideIndex)}
        </div>
      {/if}
    {/each}
  </div>
</div>
