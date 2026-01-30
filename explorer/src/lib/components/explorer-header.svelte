<script lang="ts">
import { browser } from "$app/environment"
import * as Sidebar from "$lib/components/ui/sidebar/index.js"
import Moon from "@lucide/svelte/icons/moon"
import Sun from "@lucide/svelte/icons/sun"
import { setMode } from "mode-watcher"

let { title = "Explorer" }: { title?: string } = $props()

let isDark = $state(true)

$effect(() => {
  if (browser) {
    // Check initial state
    isDark = document.documentElement.classList.contains("dark")

    // Watch for changes
    const observer = new MutationObserver(() => {
      isDark = document.documentElement.classList.contains("dark")
    })

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    })

    return () => observer.disconnect()
  }
})

function toggleMode() {
  setMode(isDark ? "light" : "dark")
}
</script>

<header class="flex h-(--header-height) shrink-0 items-center border-b border-border bg-background">
  <div class="flex w-full items-center gap-2 px-4">
    <Sidebar.Trigger class="-ms-1" />
    <div class="w-px h-4 bg-border mx-1"></div>
    <span class="text-xs font-mono uppercase tracking-wider text-muted-foreground">{title}</span>

    <div class="ms-auto flex items-center gap-3">
      <!-- Status -->
      <div class="flex items-center gap-2">
        <div class="size-1.5 bg-green-500"></div>
        <span
          class="text-[10px] font-mono text-muted-foreground uppercase tracking-wider hidden sm:inline"
        >Live</span>
      </div>

      <div class="w-px h-4 bg-border"></div>

      <!-- Mode Toggle -->
      <button
        onclick={toggleMode}
        class="p-1.5 border border-border hover:bg-muted transition-colors"
        aria-label="Toggle theme"
      >
        {#if isDark}
          <Sun class="h-3.5 w-3.5" />
        {:else}
          <Moon class="h-3.5 w-3.5" />
        {/if}
      </button>
    </div>
  </div>
</header>
