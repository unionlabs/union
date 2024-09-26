<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { goto } from "$app/navigation"
import Print from "$lib/components/Terminal/Print.svelte"
import { cn } from "$lib/utils/utils.ts"
import Button from "$lib/components/Terminal/Button.svelte"

const { contributions, terminal } = getState()

let selectedIndex = $state(0)
let buttons: Array<HTMLButtonElement> = []

function handleClick(contributor: any) {
  goto(`/0____0/${contributor.public_key_hash}`)
  terminal.setTab(4)
  terminal.setHash(contributor.public_key_hash)
}

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
$effect(() => {
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
              handleClick(contributions.data[selectedIndex])
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

  <Print>ceremony contributors</Print>
  {#each contributions.data as contributor, index}
    <Button
            bind:value={buttons[index]}
            class={cn(index === selectedIndex ? "text-union-accent-500" : "", "whitespace-nowrap text-start w-full max-w-5xl truncate")}
            onclick={() => handleClick(contributor)}
    >
      &gt {contributor.payload_id}
    </Button>
  {/each}