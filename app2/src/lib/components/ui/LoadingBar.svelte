<script lang="ts">
import { navigating } from "$app/stores"
import { cubicOut } from "svelte/easing"
import { tweened } from "svelte/motion"

type NavigationState = "loading" | "loaded" | null
let navigationState = $state<NavigationState>(null)

const progress = tweened(0, { easing: cubicOut })
let isVisible = $state(false)

let didNavigate = $state(false)

// Watch for navigation state changes
$effect(() => {
  if (navigationState === "loading") {
    didNavigate = true
    isVisible = true
    progress.set(0.7, { duration: 2000 })
  }
  if (navigationState === "loaded" && didNavigate) {
    progress.set(1, { duration: 150 })
    setTimeout(() => {
      isVisible = false
      setTimeout(() => {
        progress.set(0, { duration: 0 })
        didNavigate = false
      }, 200)
    }, 300)
  }
})

// Update navigation state based on navigating store
$effect(() => {
  navigationState = $navigating ? "loading" : "loaded"
})
</script>

<div
  class="fixed top-0 left-0 z-[999] h-[2px] w-full transition-opacity duration-200"
  class:opacity-100={isVisible}
  class:opacity-0={!isVisible}
>
  <div
    class="h-full bg-gradient-to-r from-accent to-accent/70"
    style="width: {$progress * 100}%"
  />
</div>
