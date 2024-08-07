<script lang="ts">
import { onMount } from "svelte"
import { goto } from "$app/navigation"
import { cn } from "$lib/utilities/shadcn"
import { sepoliaStore } from "$lib/wallet/evm"
import { cosmosStore } from "$lib/wallet/cosmos"
import SmileIcon from "virtual:icons/lucide/smile"
import TableIcon from "virtual:icons/lucide/table"
import BrainIcon from "virtual:icons/lucide/brain"
import { debounce } from "$lib/utilities/index.ts"
import SearchIcon from "virtual:icons/lucide/search"
import { Input } from "$lib/components/ui/input/index.ts"
import Button from "$lib/components/ui/button/button.svelte"
import * as Command from "$lib/components/ui/command/index.ts"
import DollarSignIcon from "virtual:icons/lucide/badge-dollar-sign"
import { isValidBech32Address, isValidEvmAddress } from "@union/client"

let searchInput = ""
let commandDialogOpen = false

function validAddress(address: string) {
  return isValidBech32Address(address) || isValidEvmAddress(address)
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key !== "k" || !(event.metaKey || event.ctrlKey)) return
  event.preventDefault()
  commandDialogOpen = true
}

let windowSize = { width: window.innerWidth, height: window.innerHeight }

const handleResize = () => {
  requestAnimationFrame(() => {
    windowSize = { width: window.innerWidth, height: window.innerHeight }
  })
}

onMount(() => {
  window.addEventListener("resize", handleResize)
  document.addEventListener("keydown", handleKeyDown)
  return () => {
    window.removeEventListener("resize", handleResize)
    document.removeEventListener("keydown", handleKeyDown)
  }
})

/**
 * sizes when the dialog should be open:
 * 430 or less,
 * between 960 and 768
 */
const onInputClick = (_event: MouseEvent) => {
  commandDialogOpen = windowSize.width <= 645 || (windowSize.width < 960 && windowSize.width >= 768)
}

const onInputChange = (event: InputEvent) =>
  debounce((_event: InputEvent) => {
    console.log("Searching...", searchInput)
  }, 2_500)(event)
</script>

<div class="relative mr-auto flex-1 w-full max-w-full antialiased">
  <SearchIcon class="absolute left-2.5 top-3 size-4 text-muted-foreground" />
  <Input
    type="text"
    name="search"
    autocorrect="off"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="none"
    on:click={onInputClick}
    bind:value={searchInput}
    on:input={onInputChange}
    pattern="[A-Za-z0-9\-]+"
    placeholder="Search for address, tx..."
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

<Command.Dialog
  tabindex={0}
  bind:open={commandDialogOpen}
  class={cn(
    "antialiased",
    "rounded-sm border-[1px] w-full",
    "border-solid shadow-2xl dark:border-accent/50 border-accent"
  )}
>
  <Command.Input
    tabindex={0}
    autofocus={true}
    type="text"
    name="search"
    autocorrect="off"
    inputmode="search"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="off"
    pattern="[A-Za-z0-9\-]+"
    class="my-auto h-10 lowercase"
    placeholder="Type a command or search..."
    bind:value={searchInput}
  />

  <Command.List data-search-dialog="">
    <Command.Empty class={cn("h-full py-0")}>
      {#if searchInput && searchInput?.length > 10}
        {@const isValidAddress =
          validAddress(searchInput) &&
          !(searchInput.startsWith("0x") && searchInput.length > 42)}
        <!-- TODO: this is temporary, will update tomorrow -->
        {@const isNotAddressPathParam =
          (["union", "stride", "osmosis"].some(prefix =>
            searchInput.startsWith(prefix)
          ) ||
            (searchInput.startsWith("0x") && searchInput.length > 42)) ===
          false}
        <ul class="flex flx-row justify-around size-full">
          <li class="size-full">
            <Button
              variant="link"
              disabled={!isValidAddress}
              aria-disabled={!isValidAddress}
              href={`/explorer/address/${searchInput}`}
              on:click={() => (commandDialogOpen = false)}
              class={cn(
                isValidAddress
                  ? "hover:bg-black hover:text-union-accent hover:border-union-accent"
                  : "cursor-not-allowed opacity-45 hover:bg-union-accent",
                "size-full uppercase font-mono text-xl font-semibold border border-solid border-transparent",
                "bg-union-accent text-black border-r-black border-l"
              )}
            >
              address
            </Button>
          </li>
          <li class="size-full">
            <Button
              disabled={isNotAddressPathParam}
              aria-disabled={isNotAddressPathParam}
              href={`/explorer/transfers/${searchInput}`}
              on:click={() => (commandDialogOpen = false)}
              class={cn(
                isNotAddressPathParam
                  ? "cursor-not-allowed opacity-45 hover:bg-union-accent"
                  : "hover:bg-black hover:text-union-accent hover:border-union-accent",
                "size-full uppercase font-mono text-xl font-semibold border border-solid border-transparent",
                "bg-union-accent text-black border-r-black border-l"
              )}
            >
              transaction
            </Button>
          </li>
        </ul>
      {/if}
    </Command.Empty>
    <Command.Group heading="Exploring Data">
      {@const userAddresses = [
        $sepoliaStore?.address,
        $cosmosStore?.address
      ].filter(Boolean)}
      <Command.Item
        tabindex={1}
        class={cn(
          "hover:cursor-pointer",
          userAddresses && userAddresses.length === 0 ? "hidden" : "",
          "focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1"
        )}
        onSelect={() => {
          goto(`/explorer/address/${userAddresses.join("-")}`)
          commandDialogOpen = false
        }}
      >
        <TableIcon class="mr-2 size-4" />
        <span>Your past transfers</span>
      </Command.Item>
      <Command.Item
        tabindex={2}
        class={cn(
          "hover:cursor-pointer",
          "focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1"
        )}
        onSelect={() => {
          goto(`/explorer/transfers`)
          commandDialogOpen = false
        }}
      >
        <BrainIcon class="mr-2 size-4" />
        <span>Live IBC transfer feed</span>
      </Command.Item>
    </Command.Group>
    <Command.Separator />
    <Command.Group heading="Suggestions">
      <Command.Item
        tabindex={3}
        let:attrs
        class={cn(
          "hover:cursor-pointer",
          "focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1"
        )}
        onSelect={() => {
          goto(`/transfer`)
          commandDialogOpen = false
        }}
      >
        <DollarSignIcon class="mr-2 size-4" />
        <span>Cross chain transfer</span>
      </Command.Item>
      <Command.Item
        tabindex={4}
        class={cn(
          "hover:cursor-pointer",
          "focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1"
        )}
        onSelect={() => {
          goto(`/faucet`)
          commandDialogOpen = false
        }}
      >
        <SmileIcon class="mr-2 size-4" />
        <span>Get tokens from faucet</span>
      </Command.Item>
    </Command.Group>
  </Command.List>
</Command.Dialog>

<style lang="postcss">
  /* TODO: figure out a way to style width of dialogs individually */

  :global(div[data-command-dialog-overlay], div[data-dialog-overlay]) {
    backdrop-filter: blur(2.5px);
  }
  /* :global(div[data-dialog-content]) {
    @apply rounded-lg mx-auto max-w-[450px];
  } */
</style>
