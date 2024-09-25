<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { logout } from "$lib/state/session.svelte.ts"
import Contributions from "./Contributors.svelte"
import { goto } from "$app/navigation"
import { onDestroy } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Activity from "$lib/components/Terminal/Activity.svelte"
import Blink from "$lib/components/Blink.svelte"

const { terminal } = getState()

let { children } = $props()

const changeTab = async (tab: number) => {
  if (tab === 4 && terminal.hash) {
    console.log("change tab 4", tab)
    terminal.setTab(tab)
    await goto(`/0____0/${terminal.hash}`)
  } else if (tab <= 3) {
    console.log("change tab 1, 2, 3", tab)
    terminal.setTab(tab)
    await goto("/")
  }
}

const unsubscribe = terminal.keys.subscribe(event => {
  if (event) {
    if (event.type === "keydown" && (event.shiftKey || event.ctrlKey)) {
      switch (event.key) {
        case "!": {
          changeTab(1)
          break
        }
        case "@": {
          changeTab(2)
          break
        }
        case "#": {
          changeTab(3)
          break
        }
        case "$": {
          changeTab(4)
          break
        }
        case "X": {
          logout(terminal)
          break
        }
      }
    }
  }
})

onDestroy(unsubscribe)

function autoScroll(node: HTMLElement) {
  const scroll = () => {
    node.scrollTop = node.scrollHeight
  }

  const observer = new MutationObserver(scroll)
  observer.observe(node, { childList: true, subtree: true })

  return {
    destroy() {
      observer.disconnect()
    }
  }
}
</script>

<section class="flex flex-col sm:justify-center items-center w-full h-full z-10">

  <div class="w-full h-full bg-black relative">

    <div class="w-full flex justify-between bg-neutral-800 px-4 py-2">
      <div class="flex items-center gap-2">
        <svg class="size-5" viewBox="0 0 192 192" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path fill-rule="evenodd" clip-rule="evenodd"
                d="M136 128C136 163.346 107.346 192 72 192C36.6538 192 8 163.346 8 128H40C40 145.673 54.3269 160 72 160C89.6731 160 104 145.673 104 128H136Z"
                fill="#fff"/>
          <path fill-rule="evenodd" clip-rule="evenodd" d="M40 80H8V112H40V80ZM136 80H104V112H136V80Z" fill="#fff"/>
          <path fill-rule="evenodd" clip-rule="evenodd"
                d="M152 112L184 112L184 80L152 80L152 112ZM56 112L88 112L88 80L56 80L56 112Z" fill="#fff"/>
          <path fill-rule="evenodd" clip-rule="evenodd"
                d="M56 64C56 28.6538 84.6538 -8.68512e-06 120 -5.59506e-06C155.346 -2.50499e-06 184 28.6538 184 64L152 64C152 46.3269 137.673 32 120 32C102.327 32 88 46.3269 88 64L56 64Z"
                fill="#fff"/>
        </svg>

        <button onclick={() => changeTab(1)} class="font-mono font-medium"
                class:text-white={terminal.tab !== 1}
                class:text-union-accent-500={terminal.tab === 1}
        >Tab 1
        </button>
        <button onclick={() => changeTab(2)} class="font-mono font-medium text-white"
                class:text-white={terminal.tab !== 2}
                class:text-union-accent-500={terminal.tab === 2}
        >Tab 2
        </button>
        <button onclick={() => changeTab(3)} class="font-mono font-medium text-white"
                class:text-white={terminal.tab !== 3}
                class:text-union-accent-500={terminal.tab === 3}
        >Tab 3
        </button>
        {#if terminal.hash !== undefined}
          <button class="font-mono font-medium" onclick={() => changeTab(4)}
                  class:text-white={terminal.tab !== 4}
                  class:text-union-accent-500={terminal.tab === 4}>
            {#if terminal.hash}
              {terminal.hash.slice(0, 5)}...{terminal.hash.slice(-5)}
            {:else}
              {terminal.hash}
            {/if}
          </button>
        {/if}
      </div>
      <button class="font-mono font-medium text-white" onclick={() => logout(terminal)}>Log out</button>
    </div>

    <div class="px-5 py-4 text-union-accent-50 w-full" use:autoScroll>
      {#if terminal.tab === 1}
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
      {:else if terminal.tab === 4 && terminal.hash}
        {@render children()}
      {/if}
    </div>

  </div>

</section>
