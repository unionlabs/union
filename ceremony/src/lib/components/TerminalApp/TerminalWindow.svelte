<script lang="ts">
import Print from "$lib/components/TerminalApp/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { logout } from "$lib/state/session.svelte.ts"
import Activity from "$lib/components/TerminalApp/Activity.svelte"
import { onMount } from "svelte"
import Contributions from "./Contributors.svelte"
import { goto } from "$app/navigation"

const { terminal } = getState()

let { children } = $props()

onMount(() => {
  window.addEventListener("keydown", handleCommand)

  return () => {
    window.removeEventListener("keydown", handleCommand)
  }
})

function handleCommand(event: KeyboardEvent) {
  if (event.shiftKey) {
    switch (event.key) {
      case "!": {
        changeTab(1)
        event.preventDefault()
        break
      }
      case "@": {
        changeTab(2)
        event.preventDefault()
        break
      }
      case "#": {
        changeTab(3)
        event.preventDefault()
        break
      }
      case "$": {
        changeTab(4)
        event.preventDefault()
        break
      }
      case "X": {
        logout(terminal)
        event.preventDefault()
        break
      }
    }
  }
}

const changeTab = async (tab: number) => {
  if (tab === 4 && terminal.hash) {
    terminal.setTab(tab)
    await goto(`/0____0/${terminal.hash}`)
  } else if (tab <= 3) {
    terminal.setTab(tab)
    await goto("/")
  }
}
</script>

<section class="flex flex-col sm:justify-center items-center w-full h-full p-2">
  <div class="w-full h-full max-h-[600px] max-w-4xl">
    <div class="w-full flex justify-between bg-white/50 px-3 py-2">
      <div>
        <button onclick={() => changeTab(1)} class="text-white font-mono font-medium"
                class:text-white={terminal.tab === 1}>Tab 1
        </button>
        <button onclick={() => changeTab(2)} class="text-black font-mono font-medium"
                class:text-white={terminal.tab === 2}>Tab 2
        </button>
        <button onclick={() => changeTab(3)} class="text-black font-mono font-medium"
                class:text-white={terminal.tab === 3}>Tab 3
        </button>
        {#if terminal.hash !== undefined}
          <button class="text-black font-mono font-medium " onclick={() => changeTab(4)}
                  class:text-white={terminal.tab === 4}>{terminal.hash}</button>
        {/if}
      </div>
      <button class="text-black font-mono font-medium" onclick={() => logout(terminal)}>Log out</button>
    </div>
    <div class="border-[4px] border-white/50 px-5 py-4 text-union-accent-50 h-full w-ful overflow-y-auto overflow-hiddenl">

      {#if terminal.tab === 1 }
        <div class="flex flex-col">
          {#each terminal.history as text}
            <Print>{text}</Print>
          {/each}
        </div>
        {@render children()}

      {:else if terminal.tab === 2}
        <Activity/>

      {:else if terminal.tab === 3}
        <Contributions/>

      {:else if terminal.tab === 4}
        {@render children()}

      {/if}
    </div>
  </div>
</section>
