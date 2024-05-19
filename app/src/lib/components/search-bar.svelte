<script lang="ts">
import { onMount } from "svelte"
import { goto } from "$app/navigation"
import Smile from "virtual:icons/lucide/smile"
import Table from "virtual:icons/lucide/table"
import Brain from "virtual:icons/lucide/brain"
import Search from "virtual:icons/lucide/search"
import { debounce } from "$lib/utilities/index.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import * as Command from "$lib/components/ui/command/index.ts"
import DollarSign from "virtual:icons/lucide/badge-dollar-sign"

let commandDialogOpen = false
let searchInput: string

function handleKeyDown(event: KeyboardEvent) {
  if (event.key !== "k" || !(event.metaKey || event.ctrlKey)) return
  event.preventDefault()
  commandDialogOpen = true
}

let windowSize = { width: window.innerWidth, height: window.innerHeight }
const handleResize = () => (windowSize = { width: window.innerWidth, height: window.innerHeight })

onMount(() => {
  window.addEventListener("resize", handleResize)
  document.addEventListener("keydown", handleKeyDown)
  return () => {
    window.removeEventListener("resize", handleResize)
    document.removeEventListener("keydown", handleKeyDown)
  }
})

const onInputClick = (event: MouseEvent) => (commandDialogOpen = windowSize.width < 720)
const onInputChange = (event: InputEvent) =>
  debounce((event: InputEvent) => {
    console.log("Searching...", searchInput)
  }, 1_000)(event)
</script>

<div class="relative mr-auto flex-1 w-full max-w-[475px]">
  <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
  <Input
    type="search"
    name="search"
    pattern="[a-z]"
    autocorrect="off"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="none"
    placeholder="Search..."
    on:click={onInputClick}
    bind:value={searchInput}
    on:input={onInputChange}
    class="w-full rounded-lg bg-background pl-8 self-stretch lowercase border-[1px] bprder-[#fafafa25]/10"
  />
  <kbd
    class="absolute right-2.5 top-2.5 pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium text-muted-foreground opacity-100"
  >
    <span class="text-xs">⌘</span>K
  </kbd>
</div>

<Command.Dialog
  bind:open={commandDialogOpen}
  class="rounded-lg border-[1.5px] border-solid shadow-2xl border-accent w-full"
>
  <Command.Input
    type="search"
    pattern="[a-z]"
    autocorrect="off"
    spellcheck="false"
    autocapitalize="off"
    class="my-auto h-9 lowercase"
    placeholder="Type a command or search..."
  />

  <Command.List data-search-dialog="">
    <Command.Empty>No results found.</Command.Empty>
    <Command.Group heading="Suggestions">
      <Command.Item
        class="hover:cursor-pointer"
        onSelect={() => {
          goto(`/send`)
          commandDialogOpen = false
        }}
      >
        <DollarSign class="mr-2 size-4" />
        <span>Send & Swap</span>
      </Command.Item>
      <Command.Item
        class="hover:cursor-pointer"
        onSelect={() => {
          goto(`/faucet`)
          commandDialogOpen = false
        }}
      >
        <Smile class="mr-2 size-4" />
        <span>Get tokens from faucet</span>
      </Command.Item>
    </Command.Group>
    <Command.Separator />
    <Command.Group heading="Exploring Data">
      <Command.Item>
        <Brain class="mr-2 size-4" />
        <span>Investigate IBC activity</span>
      </Command.Item>
      <Command.Item
        class="hover:cursor-pointer"
        onSelect={() => {
          goto(`/transfers`)
          commandDialogOpen = false
        }}
      >
        <Table class="mr-2 size-4" />
        <span>View your past transfers</span>
      </Command.Item>
    </Command.Group>
  </Command.List>
</Command.Dialog>

<style lang="postcss">
  /* TODO: figure out a way to style width of dialogs individually */

  /* :global(div[data-dialog-content]) {
    @apply rounded-lg mx-auto max-w-[450px];
  } */
</style>
