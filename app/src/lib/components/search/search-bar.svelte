<script lang="ts">
import { cn } from "$lib/utilities/shadcn"
import SearchIcon from "virtual:icons/lucide/search"
import { Input } from "$lib/components/ui/input/index.ts"

export let searchInput = ""
export let onInputClick: ((event: MouseEvent) => void) | undefined = undefined
export let onInputChange: ((event: InputEvent) => void) | undefined = undefined
export let windowWidth = window.innerWidth
</script>

<div class="relative mr-auto flex-1 w-full max-w-full antialiased">
  <SearchIcon class="absolute left-2.5 top-2.5 size-5 text-muted-foreground" />
  <Input
    type="text"
    name="search"
    autocorrect="off"
    inputmode="search"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="off"
    on:click={onInputClick}
    on:input={onInputChange}
    bind:value={searchInput}
    pattern="[A-Za-z0-9\-]+"
    placeholder={(windowWidth >= 930 || windowWidth <= 768) && windowWidth > 538
      ? "Search for address or tx hash..."
      : "Search..."}
    class={cn(
      "h-10",
      "shadow-sm transition-colors placeholder:text-muted-foreground",
      "w-full bg-background pl-8 self-stretch lowercase border-[1px] border-input",
      "focus-visible:border-secondary focus-visible:outline-none focus-visible:ring-0 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
    )}
  />
  <kbd
    class={cn(
      "h-7 gap-0.5 px-1.5",
      "text-white dark:text-black",
      "absolute select-none pointer-events-none",
      "right-1.5 top-1.5 inline-flex items-center border bg-primary font-mono text-xs font-medium opacity-100"
    )}
  >
    <span class="text-sm mb-1"><span class="text-lg mr-0.25">⌘</span>K</span>
  </kbd>
</div>
