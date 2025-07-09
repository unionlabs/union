<script lang="ts">
import { page } from "$app/state"
import { uiStore } from "$lib/stores/ui.svelte"
import type { NavItem } from "./config"

let {
  item,
  expanded = false,
  isExpanded = false,
  onToggle,
}: {
  item: NavItem
  expanded?: boolean
  isExpanded?: boolean
  onToggle: () => void
} = $props()

const isSubrouteActive = $derived(
  item.subroutes?.some(sub => page.url.pathname === sub.path) ?? false,
)
</script>

<div>
  <button
    onclick={onToggle}
    class="inline-flex items-center rounded-lg hover:bg-zinc-800 hover:text-zinc-200 w-full"
    class:text-accent={isSubrouteActive}
    class:text-zinc-400={!isSubrouteActive}
  >
    <span class="relative inline-block">
      <div class="m-2 flex h-8 w-8 items-center justify-center flex-shrink-0">
        <svelte:component
          this={item.icon}
          class="size-5 {isSubrouteActive ? 'text-accent' : 'text-zinc-500'}"
        />
      </div>
    </span>
    <div class="flex w-full items-center justify-between whitespace-nowrap px-2">
      <p>{item.title}</p>
      {#if expanded}
        <svg
          class="w-4 h-4 transition-transform duration-200 text-zinc-500"
          class:rotate-90={isExpanded}
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fill-rule="evenodd"
            d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
            clip-rule="evenodd"
          />
        </svg>
      {/if}
    </div>
  </button>

  {#if expanded && isExpanded && item.subroutes && item.subroutes.length > 0}
    <div class="ml-8 mt-1 space-y-0.5">
      {#each item.subroutes as subroute}
        {#if !subroute.editions || subroute.editions.includes(uiStore.edition)}
          <a
            href={subroute.path}
            class="block py-1.5 px-2 text-xs text-zinc-500 hover:text-zinc-300 rounded transition-colors duration-200"
            class:text-accent={page.url.pathname === subroute.path}
            class:bg-zinc-800={page.url.pathname === subroute.path}
          >
            <span>{subroute.title}</span>
            {#if subroute.new}
              <span
                class="ml-1 inline-flex items-center px-1 py-0.5 rounded text-xs font-medium bg-accent/20 text-accent"
              >
                NEW
              </span>
            {/if}
          </a>
        {/if}
      {/each}
    </div>
  {/if}
</div>
