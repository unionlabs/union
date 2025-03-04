<script lang="ts">
import Wallet from "$lib/components/ui/Wallet/index.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import ConnectWalletButton from "$lib/components/ui/ConnectWalletButton.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "$lib/components/ui/Button.svelte"
import SharpSettingsIcon from "$lib/components/icons/SharpSettingsIcon.svelte"
import { page } from "$app/stores"
import { cn } from "$lib/utils"
import { onMount } from "svelte"
import { navigation } from "./navigation"

const isCurrentPath = (path: string) => $page.url.pathname === path

let activeElement: HTMLElement | null = null
let highlightElement: HTMLElement

$: if ($page.url.pathname) {
  updateHighlightPosition()
}

const updateHighlightPosition = () => {
  if ($page.url.pathname && highlightElement) {
    const newActive = document.querySelector(`[data-path="${$page.url.pathname}"]`) as HTMLElement
    if (newActive) {
      const rect = newActive.getBoundingClientRect()
      highlightElement.style.top = `${rect.top}px`
      highlightElement.style.left = `${rect.left}px`
      highlightElement.style.width = `${rect.width}px`
      highlightElement.style.height = `${rect.height}px`
      highlightElement.style.opacity = "1"
      activeElement = newActive
    } else {
      highlightElement.style.opacity = "0"
      activeElement = null
    }
  }
}

onMount(() => {
  window.addEventListener('resize', updateHighlightPosition)
  // Trigger initial position
  updateHighlightPosition()

  return () => {
    window.removeEventListener('resize', updateHighlightPosition)
  }
})
</script>

<div class="relative h-full">
  <div
    bind:this={highlightElement}
    class="absolute -z-10 bg-zinc-200 dark:bg-zinc-700 rounded-lg transition-all duration-300"
  ></div>


<div class="p-6 min-h-full flex flex-col overflow-y-auto">
  <img class="self-start h-12 mb-6" src="/images/union-logo.svg" alt="Union" />
  <div class="flex flex-col justify-between flex-1">
  {#each navigation as section}
    <section>
      {#if section.title}
        <h2 class="font-bold text-xl mb-2">{section.title}</h2>
      {/if}
      <ul class="flex flex-col gap-1">
        {#each section.items as item}
          <li>
            <a 
              href={item.path} 
              data-path={item.path}
              class={cn(
                "relative flex items-center gap-2 px-3 py-2 rounded-lg transition-colors",
                isCurrentPath(item.path) ? "" : "hover:bg-zinc-100 dark:hover:bg-zinc-800"
              )}
            >
              <svelte:component 
                this={item.icon} 
                class="size-5 text-zinc-600 dark:text-zinc-400" 
              />
              {item.title}
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
  </div>

  <div class="flex flex-col gap-2">
    <ConnectWalletButton/>
    <Button variant="secondary" onclick={() => uiStore.openSettingsModal()}>
      <SharpSettingsIcon class="size-5"/>
      Settings
    </Button>
  </div>

</div>
</div>
