<script lang="ts">
import type { DetectedOS } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import type { KeyEvent } from "$lib/state/terminal.svelte.ts"

type Props = {
  select: (os: DetectedOS) => void
}

const { terminal } = getState()

let { select }: Props = $props()
let showButtons = $state(true)

const selections: Array<DetectedOS> = ["Linux", "macOS"]
let currentFocusIndex = $state(0)

onMount(() => {
  terminal.updateHistory("Select your OS", { duplicate: true })
})

function handleKeyDown(event: KeyEvent) {
  if (event.type === "keydown") {
    switch (event.key) {
      case "ArrowUp":
        currentFocusIndex = (currentFocusIndex - 1 + selections.length) % selections.length
        break
      case "ArrowDown":
        currentFocusIndex = (currentFocusIndex + 1) % selections.length
        break
      case "Enter": {
        showButtons = false
        select(selections[currentFocusIndex])
        break
      }
    }
  }
}

const unsubscribe = terminal.keys.subscribe(event => {
  if (event) {
    handleKeyDown(event)
  }
})

onDestroy(unsubscribe)
</script>


{#if showButtons}
  {#each selections as os, index}
    <button
            class="block outline-none focus:ring-2 focus:ring-transparent focus:border-none"
            class:text-union-accent-500={currentFocusIndex === index}
            onclick={() => select(os)}
    >
      &gt {os}
    </button>
  {/each}
{/if}
