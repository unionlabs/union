<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { goto } from "$app/navigation"
import Print from "$lib/components/Terminal/Print.svelte"
import { cn } from "$lib/utils/utils.ts"
import Button from "$lib/components/Terminal/Button.svelte"
import { onMount } from "svelte"

const { contributions, terminal } = getState()

let focusedIndex = $state(0)
let buttons: Array<HTMLButtonElement> = []

function handleClick(contributor: any) {
  goto(`/0____0/${contributor.public_key_hash}`)
  terminal.setTab(4)
  terminal.setHash(contributor.public_key_hash)
}

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if (event.type === "keydown") {
          if (event.key === "ArrowUp") {
            focusedIndex =
              (focusedIndex - 1 + contributions.data.length) % contributions.data.length
            buttons[focusedIndex]?.focus()
          } else if (event.key === "ArrowDown") {
            focusedIndex = (focusedIndex + 1) % contributions.data.length
            buttons[focusedIndex]?.focus()
          } else if (event.key === "Enter") {
            if (buttons[focusedIndex]) {
              handleClick(contributions.data[focusedIndex])
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

  {#each contributions.data as contributor, index}
    <Button
            bind:value={buttons[index]}
            onmouseenter={() => focusedIndex = index}
            class={cn(index === focusedIndex ? "bg-union-accent-500 text-black" : "", "whitespace-nowrap text-start w-fit max-w-5xl truncate")}
            onclick={() => handleClick(contributor)}
    >
      &gt {contributor.payload_id}
    </Button>
  {/each}