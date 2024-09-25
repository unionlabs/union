<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { onMount } from "svelte"
import { goto } from "$app/navigation"
import Print from "$lib/components/Terminal/Print.svelte"

const { contributions, terminal } = getState()

let selectedIndex = $state(0)
let buttons: Array<HTMLButtonElement> = []

function fireEvent(contributor: any) {
  console.log("selected contributor:", contributor)
  goto(`/0____0/${contributor.public_key_hash}`)
  terminal.setTab(4)
  terminal.setHash(contributor.public_key_hash)
}

onMount(() => {
  buttons[0].focus()
})

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if (event.type === "keydown") {
          if (event.key === "ArrowUp") {
            selectedIndex =
              (selectedIndex - 1 + contributions.data.length) % contributions.data.length
            buttons[selectedIndex]?.focus()
          } else if (event.key === "ArrowDown") {
            selectedIndex = (selectedIndex + 1) % contributions.data.length
            buttons[selectedIndex]?.focus()
          } else if (event.key === "Enter") {
            if (buttons[selectedIndex]) {
              fireEvent(contributions.data[selectedIndex])
            }
          }
        }
      }
    })
  }, 200)
  return () => {
    if (subscriptionTimeout) {
      clearTimeout(subscriptionTimeout)
    }
    if (unsubscribe) {
      unsubscribe()
    }
  }
})
</script>

{#if contributions.data}
  <Print>ceremony contributors</Print>
  {#each contributions.data as contributor, index}
    <button
            bind:this={buttons[index]}
            class="text-start w-full max-w-5xl whitespace-nowrap truncate focus:outline-none"
            class:text-union-accent-500={index === selectedIndex}
    >
      &gt {contributor.payload_id}
    </button>
  {/each}
{/if}