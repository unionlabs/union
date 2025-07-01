<script lang="ts">
import { page } from "$app/state"
import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte"
import ConnectWalletButton from "$lib/components/ui/ConnectWalletButton.svelte"
import ProfileCard from "$lib/dashboard/components/SideCard.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { cn } from "$lib/utils"
import { onMount } from "svelte"
import { navigation } from "./navigation.js"

const isCurrentPath = (path: string) => {
  // Exact match
  if (page.url.pathname === path) {
    return true
  }
  // Check if current path is a subroute of the navigation item
  // For example, /explorer/packets/123 should highlight /explorer/packets
  if (path !== "/" && page.url.pathname.startsWith(`${path}/`)) {
    return true
  }

  return false
}

let highlightElement: HTMLElement
let overflowContainer: HTMLDivElement

const updateHighlightPosition = () => {
  const yOffset = overflowContainer?.scrollTop ?? 0
  if (page.url.pathname && highlightElement) {
    // Find the best matching navigation item
    let bestMatch: HTMLElement | null = null
    let bestMatchLength = 0

    // Check all navigation items to find the best match
    const allNavItems = document.querySelectorAll("[data-path]")
    allNavItems.forEach((item) => {
      const itemPath = item.getAttribute("data-path")
      if (
        itemPath
        && (page.url.pathname === itemPath
          || (page.url.pathname.startsWith(`${itemPath}/`)
            && itemPath.length > bestMatchLength))
      ) {
        bestMatch = item as HTMLElement
        bestMatchLength = itemPath.length
      }
    })

    if (bestMatch) {
      const rect = (bestMatch as HTMLElement).getBoundingClientRect()
      highlightElement.style.top = `${rect.top + yOffset}px`
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
  if (page.url.pathname) {
    updateHighlightPosition()
  }
})

onMount(() => {
  window.addEventListener("resize", updateHighlightPosition)
  // Use requestAnimationFrame to ensure DOM is fully rendered
  requestAnimationFrame(() => {
    updateHighlightPosition()
  })

  return () => {
    window.removeEventListener("resize", updateHighlightPosition)
  }
})
</script>

<div
  class="relative h-full overflow-y-auto"
  bind:this={overflowContainer}
>
  <div
    bind:this={highlightElement}
    class="absolute -z-10 bg-gradient-to-r from-accent/10 to-accent/5 border border-accent/20 rounded-lg transition-all duration-300 ease-out shadow-sm shadow-accent/10"
  >
  </div>

  <div class="min-h-full flex flex-col overflow-y-auto">
    <a
      class="px-6 flex items-center gap-2 border-b-1 h-16 border-zinc-900"
      href="/"
    >
      <img
        class="h-10"
        src="/images/union-logo.svg"
        alt="Union"
      />
      {#key uiStore.edition}
        {#if uiStore.edition === "app"}
          <div class="flex items-center gap-1 bg-accent/10 border border-accent/20 px-1.5 py-1 rounded text-accent hover:bg-accent/20 transition-all duration-200">
            <span class="text-sm font-mono font-bold leading-none">{uiStore.theme.label}</span>
          </div>
        {:else}
          <div class="bg-accent px-2 py rounded text-sm font-mono font-bold">
            {uiStore.theme.label}
          </div>
        {/if}
      {/key}
    </a>
    <div class="flex flex-col flex-1">
      <ProfileCard />
      {#each navigation as section}
        {#if section.title !== "Developer" || uiStore.showDeveloperPages}
          {#if section.title === "More Union"}
            <!-- Special rendering for More Union section - just icons in a row -->
            <section class="px-4 py-4 last:flex-1 flex flex-col justify-end">
              <div class="flex items-center justify-center gap-4">
                {#each section.items as item}
                  <a
                    href={item.path}
                    target={item.external ? "_blank" : undefined}
                    rel={item.external ? "noopener noreferrer" : undefined}
                    class="flex items-center justify-center w-10 h-10 rounded-lg bg-zinc-900/50 hover:bg-zinc-800 transition-all duration-200 text-zinc-400 hover:text-zinc-200"
                    title={item.title}
                  >
                    <item.icon class="w-5 h-5" />
                  </a>
                {/each}
              </div>
            </section>
          {:else}
            <!-- Regular navigation section rendering -->
            <section class="px-4 py-4 last:flex-1 flex flex-col justify-end">
              {#if section.title}
                <h2 class="font-semibold text-xs mb-4 text-left uppercase tracking-wider text-zinc-500 px-3">
                  {section.title}
                </h2>
              {/if}
              <ul class="flex flex-col gap-0.5">
                {#each section.items as item}
                  <li>
                    <a
                      href={item.path}
                      data-path={item.path}
                      target={item.external ? "_blank" : undefined}
                      rel={item.external ? "noopener noreferrer" : undefined}
                      class={cn(
                        "relative flex items-center gap-3 px-3 py-2.5 mx-1 rounded-lg transition-all duration-200",
                        isCurrentPath(item.path)
                          ? "text-white font-medium"
                          : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/50",
                      )}
                    >
                      <item.icon
                        class={cn(
                          "w-5 h-5 transition-colors duration-200",
                          isCurrentPath(item.path)
                            ? "text-accent"
                            : "text-zinc-500",
                        )}
                      />
                      <span class="text-sm font-medium">{item.title}</span>
                      {#if item.external}
                        <ExternalLinkIcon class="w-4 h-4 ml-auto text-zinc-500" />
                      {/if}
                    </a>

                    {#if item.subroutes && item.subroutes.length > 0}
                      <ul class="flex flex-col gap-0.5 mt-1 ml-6 border-l border-zinc-800/50 pl-4">
                        {#each item.subroutes as subroute}
                          {#if !subroute.editions || subroute.editions.includes(uiStore.edition)}
                            <li>
                              <a
                                href={subroute.path}
                                data-path={subroute.path}
                                class={cn(
                                  "relative flex items-center gap-2 px-3 py-2 rounded-md transition-all duration-200",
                                  isCurrentPath(subroute.path)
                                    ? "text-white font-medium bg-zinc-800/20"
                                    : "text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900/30",
                                )}
                              >
                                <span class="text-sm">{subroute.title}</span>
                              </a>
                            </li>
                          {/if}
                        {/each}
                      </ul>
                    {/if}
                  </li>
                {/each}
              </ul>
            </section>
          {/if}
        {/if}
      {/each}
    </div>

    <div class="p-4 border-t border-zinc-800/50 bg-zinc-950/30">
      <ConnectWalletButton />
      <!--
       <Button variant="secondary" onclick={() => uiStore.openSettingsModal()}>
         <SharpSettingsIcon class="size-5"/>
         Settings
       </Button>
       !-->
    </div>
  </div>
</div>
