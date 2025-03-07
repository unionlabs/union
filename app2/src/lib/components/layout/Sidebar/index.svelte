<script lang="ts">
import ConnectWalletButton from "$lib/components/ui/ConnectWalletButton.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Button from "$lib/components/ui/Button.svelte"
import SharpSettingsIcon from "$lib/components/icons/SharpSettingsIcon.svelte"
import { page } from "$app/stores"
import { cn } from "$lib/utils"
import { onMount } from "svelte"
import { navigation } from "./navigation.js"

const isCurrentPath = (path: string) => $page.url.pathname === path

let highlightElement: HTMLElement

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
    } else {
      highlightElement.style.opacity = "0"
    }
  }
}

$effect(() => {
  if ($page.url.pathname) {
    updateHighlightPosition()
  }
})

onMount(() => {
  window.addEventListener("resize", updateHighlightPosition)
  // Trigger initial position
  updateHighlightPosition()

  return () => {
    window.removeEventListener("resize", updateHighlightPosition)
  }
})
</script>

<div class="relative h-full">
  <div
    bind:this={highlightElement}
    class="absolute -z-10 bg-zinc-800 rounded-lg transition-all duration-300"
  ></div>


<div class="min-h-full flex flex-col overflow-y-auto">
  <div class="px-6 flex items-center border-b-1 h-16 border-zinc-900">
    <img class="h-10" src="/images/union-logo.svg" alt="Union" />
  </div>
  <div class="flex flex-col justify-between flex-1">
  {#each navigation as section}
    <section class="border-zinc-900 p-6">
      {#if section.title}
        <h2 class="font-bold text-sm -mt-8.5 mb-2.5 text-center uppercase text-zinc-600">{section.title}</h2>
      {/if}
      <ul class="flex flex-col gap-1">
        {#each section.items as item}
          <li>
            <a 
              href={item.path} 
              data-path={item.path}
              class={cn(
                "relative flex items-center gap-2 px-3 py-2 rounded-lg transition-colors",
                isCurrentPath(item.path) ? "" : "dark:hover:bg-zinc-900"
              )}
            >
              <svelte:component 
                this={item.icon} 
                class="size-5 text-zinc-500" 
              />
              {item.title}
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/each}
  </div>

  <div class="flex flex-col gap-2 p-6 border-t border-zinc-900">
    <ConnectWalletButton/>
    <Button variant="secondary" onclick={() => uiStore.openSettingsModal()}>
      <SharpSettingsIcon class="size-5"/>
      Settings
    </Button>
  </div>

</div>
</div>
