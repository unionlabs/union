<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import {onDestroy, onMount} from "svelte"
import Print from "$lib/components/TerminalApp/Print.svelte"
import { goto } from "$app/navigation"

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

const unsubscribe = terminal.keys.subscribe((event) => {
  if (event) {
    if (!contributions.data) return
    if(event.type !== 'keydown') return;
    console.log('dd')
    switch (event.key) {
      case "ArrowUp": {
        selectedIndex = (selectedIndex - 1 + contributions.data.length) % contributions.data.length
        buttons[selectedIndex]?.focus()
        break
      }
      case "ArrowDown": {
        selectedIndex = (selectedIndex + 1) % contributions.data.length
        buttons[selectedIndex]?.focus()
        break
      }
      case "Enter": {
        if (buttons[selectedIndex]) {
          fireEvent(contributions.data[selectedIndex])
        }
        break
      }
    }
  }
});

onDestroy(unsubscribe);

</script>

{#if contributions.data}
  <Print>ceremony contributors</Print>
  {#each contributions.data as contributor, index}
    <button
            bind:this={buttons[index]}
            class="text-start w-full max-w-5xl whitespace-nowrap truncate focus:outline-none"
            class:text-union-accent-500={index === selectedIndex}
            onclick={() => fireEvent(contributor)}
    >
      &gt {contributor.payload_id}
    </button>
  {/each}
{/if}