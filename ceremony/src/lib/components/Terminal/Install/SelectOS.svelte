<script lang="ts">
import { cn, type DetectedOS } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import type { KeyEvent } from "$lib/state/terminal.svelte.ts"
import Button from "$lib/components/Terminal/Button.svelte"

type Props = {
  select: (os: DetectedOS) => void
}

const { terminal } = getState()

let { select }: Props = $props()
let showButtons = $state(true)

const selections: Array<DetectedOS> = ["Linux", "macOS"]
let focusedIndex = $state(0)

onMount(() => {
  terminal.updateHistory("No MPC client detected", { duplicate: true })
  terminal.updateHistory("Select your OS for instructions", { duplicate: true })
})

function handleKeyDown(event: KeyEvent) {
  if (event.type === "keydown") {
    if (event.key === "ArrowUp") {
      focusedIndex = (focusedIndex - 1 + selections.length) % selections.length
    } else if (event.key === "ArrowDown") {
      focusedIndex = (focusedIndex + 1) % selections.length
    } else if (event.key === "Enter") {
      showButtons = false
      select(selections[focusedIndex])
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
    <Button
            onmouseenter={() => focusedIndex = index}
            class={cn(index === focusedIndex ? "bg-union-accent-500 text-black" : "")}
            onclick={() => select(os)}
    >
      &gt {os}
    </Button>
  {/each}
{/if}
