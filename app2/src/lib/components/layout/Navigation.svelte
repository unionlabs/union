<script lang="ts">
import { page } from "$app/stores"
import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import { cn } from "$lib/utils"
import { onMount } from "svelte"
import { navigation } from "./Sidebar/navigation"
import YapAd from "./YapAd.svelte"

interface Props {
  onItemClick?: () => void
  class?: string
  variant?: "animated" | "static"
}

const {
  onItemClick,
  class: className = "",
  variant = "static",
}: Props = $props()

let highlightElement: HTMLElement | undefined
let navigationContainer: HTMLDivElement | undefined

const updateHighlightPosition = () => {
  if (variant === "animated" && $page.url.pathname && highlightElement && navigationContainer) {
    let bestMatch: HTMLElement | null = null
    let bestMatchLength = 0

    const allNavItems = navigationContainer.querySelectorAll("[data-path]")
    allNavItems.forEach((item) => {
      const itemPath = item.getAttribute("data-path")
      if (
        itemPath
        && ($page.url.pathname === itemPath
          || ($page.url.pathname.startsWith(`${itemPath}/`)
            && itemPath.length > bestMatchLength))
      ) {
        bestMatch = item as HTMLElement
        bestMatchLength = itemPath.length
      }
    })

    if (bestMatch) {
      const rect = (bestMatch as HTMLElement).getBoundingClientRect()
      const containerRect = navigationContainer.getBoundingClientRect()

      if (containerRect.width > 0 && containerRect.height > 0) {
        highlightElement.style.top = `${rect.top - containerRect.top}px`
        highlightElement.style.left = `${rect.left - containerRect.left}px`
        highlightElement.style.width = `${rect.width}px`
        highlightElement.style.height = `${rect.height}px`
        highlightElement.style.opacity = "1"
      }
    } else {
      highlightElement.style.opacity = "0"
    }
  }
}

$effect(() => {
  if ($page.url.pathname && variant === "animated") {
    updateHighlightPosition()
    setTimeout(updateHighlightPosition, 100)
  }
})

onMount(() => {
  if (variant === "animated") {
    window.addEventListener("resize", updateHighlightPosition)
    requestAnimationFrame(() => {
      updateHighlightPosition()
    })

    return () => {
      window.removeEventListener("resize", updateHighlightPosition)
    }
  }
})

const isCurrentPath = (path: string) => {
  if ($page.url.pathname === path) {
    return true
  }

  for (const section of navigation) {
    for (const item of section.items) {
      if (item.path === path) {
        if (item.subroutes) {
          for (const subroute of item.subroutes) {
            if (
              $page.url.pathname === subroute.path
              || (subroute.path !== "/" && $page.url.pathname.startsWith(`${subroute.path}/`))
            ) {
              return true
            }
          }
        }
        if (path !== "/" && $page.url.pathname.startsWith(`${path}/`)) {
          return true
        }
        break
      }

      if (item.subroutes) {
        for (const subroute of item.subroutes) {
          if (subroute.path === path) {
            if (path !== "/" && $page.url.pathname.startsWith(`${path}/`)) {
              return true
            }
            break
          }
        }
      }
    }
  }

  return false
}

const hasActiveSubroute = (item: any) => {
  if (!item.subroutes) {
    return false
  }

  return item.subroutes.some((subroute: any) =>
    $page.url.pathname === subroute.path
    || (subroute.path !== "/" && $page.url.pathname.startsWith(`${subroute.path}/`))
  )
}

let isMoreUnionFirst = $derived(
  navigation.findIndex(section => section.title === "More Union") === 0,
)
</script>

<div
  class={cn("flex flex-col relative", className)}
  bind:this={navigationContainer}
>
  {#if variant === "animated"}
    <div
      bind:this={highlightElement}
      class="absolute -z-10 bg-gradient-to-r from-accent/20 to-accent/10 border border-accent/30 rounded-lg transition-all duration-300 ease-out shadow-sm shadow-accent/20 pointer-events-none"
    >
    </div>
  {/if}

  {#each navigation as section, index}
    {#if section.title !== "Developer" || uiStore.showDeveloperPages}
      {#if section.title === "More Union"}
        <!-- Spacer to push social icons to bottom -->
        <div class="flex-1"></div>
        <YapAd />
        <!-- Special rendering for More Union section - just icons in a row -->
        <section class="px-4 py-4 flex flex-col justify-end {!isMoreUnionFirst && index > 0 ? 'border-t border-zinc-800/50' : ''}">
          <div class="flex items-center justify-center gap-4">
            {#each section.items as item}
              <a
                href={item.path}
                target={item.external ? "_blank" : undefined}
                rel={item.external ? "noopener noreferrer" : undefined}
                class="flex items-center justify-center w-10 h-10 rounded-lg bg-zinc-900/50 hover:bg-zinc-800 transition-all duration-200 text-zinc-400 hover:text-zinc-200"
                title={item.title}
                onclick={onItemClick}
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
                      ? variant === "static" && !hasActiveSubroute(item)
                        ? "text-white font-medium bg-gradient-to-r from-accent/20 to-accent/10 border border-accent/30"
                        : "text-white font-medium"
                      : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900/50",
                  )}
                  onclick={onItemClick}
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
                                ? variant === "static"
                                  ? "text-white font-medium bg-gradient-to-r from-accent/20 to-accent/10 border border-accent/30"
                                  : "text-white font-medium bg-zinc-800/20"
                                : "text-zinc-500 hover:text-zinc-300 hover:bg-zinc-900/30",
                            )}
                            onclick={onItemClick}
                          >
                            <span class="text-sm">{subroute.title}</span>
                            {#if subroute.new}
                              <span
                                class="ml-auto bg-accent/10 border border-accent/20 px-1.5 py-0.5 rounded text-accent text-xs font-mono font-bold leading-none"
                              >
                                NEW
                              </span>
                            {/if}
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
