<script>
import { createEventDispatcher, onMount, onDestroy } from "svelte"

const dispatch = createEventDispatcher()

function handleKeydown(event) {
  dispatch("key", {
    key: event.key,
    code: event.code,
    ctrlKey: event.ctrlKey,
    shiftKey: event.shiftKey,
    altKey: event.altKey,
    metaKey: event.metaKey
  })

  switch (event.code) {
    case "Enter":
      dispatch("Enter", event)
      break
    case "Space":
      dispatch("Space", event)
      break
    // Add more cases for other specific keys as needed
  }
}

onMount(() => {
  window.addEventListener("keydown", handleKeydown)
})

onDestroy(() => {
  window.removeEventListener("keydown", handleKeydown)
})
</script>