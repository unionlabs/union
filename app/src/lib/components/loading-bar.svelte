<script lang="ts">
import { tweened } from "svelte/motion"
import { cubicOut } from "svelte/easing"
import { navigating } from "$app/stores"
import { writable } from "svelte/store"

type NavigationState = "loading" | "loaded" | null
const navigationState = writable<NavigationState>(null)

const progress = tweened(0, { easing: cubicOut })

let didNavigate = false

navigationState.subscribe(state => {
  if (state === "loading") {
    didNavigate = true
    progress.set(0.7, { duration: 2000 })
  }
  if (state === "loaded" && didNavigate) {
    progress.set(1, { duration: 300 })
    setTimeout(() => {
      progress.set(0, { duration: 0 })
    }, 300)
  }
})

$: $navigationState = $navigating ? "loading" : "loaded"
</script>

<div class="absolute top-0 z-[999] h-[2px] w-full">
  <div class="h-full bg-union-accent-600 transition" style="width: {$progress * 100}%" />
</div>