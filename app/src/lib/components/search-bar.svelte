<script lang="ts">
import {
  isValidEvmTxHash,
  isValidEvmAddress,
  isValidCosmosTxHash,
  isValidBech32Address
} from "@union/client"
import { onMount } from "svelte"
import { page } from "$app/stores"
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
import Badge from "$lib/components/ui/badge/badge.svelte"
import * as Command from "$lib/components/ui/command/index.ts"
import DollarSignIcon from "virtual:icons/lucide/badge-dollar-sign"

let searchInput = ""
$: searchInput = searchInput.replaceAll(" ", "")

let commandDialogOpen = false

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
  debounce((_event: InputEvent) => console.log("Searching...", searchInput), 1_500)(event)

function validTxHash(hash: string) {
  return isValidCosmosTxHash(hash) || isValidEvmTxHash(hash)
}

function validAddress(address: string) {
  return isValidBech32Address(address) || isValidEvmAddress(address)
}

function onEnterPress(event: KeyboardEvent) {
  event.stopPropagation()
  if (event.key === "Escape") commandDialogOpen = false
  if (event.key !== "Enter") return

  if (validTxHash(searchInput)) {
    goto(`/explorer/transfers/${searchInput}`)
    commandDialogOpen = false
    searchInput = ""
  }

  let input = searchInput.includes("-") ? searchInput.split("-") : [searchInput]

  input = input.filter(validAddress)

  if (input.length > 0) {
    goto(`/explorer/address/${input.join("-")}`)
    commandDialogOpen = false
    searchInput = ""
  }

  console.log("Searching...", searchInput)
}
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
    on:keydown={onEnterPress}
    pattern="[A-Za-z0-9\-]+"
    placeholder={windowSize.width >= 960
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

<Command.Dialog
  tabindex={0}
  label="Search Dialog"
  onKeydown={onEnterPress}
  bind:open={commandDialogOpen}
  class={cn(
    "rounded-sm border-[1px] w-full",
    "border-solid shadow-2xl dark:border-accent/50 border-accent"
  )}
>
  <Command.Input
    tabindex={0}
    type="text"
    name="search"
    autofocus={true}
    autocorrect="off"
    inputmode="search"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="off"
    bind:value={searchInput}
    pattern="[A-Za-z0-9\-]+"
    placeholder="Navigate, search for address or tx by hash..."
    class="my-auto h-10 lowercase placeholder:text-xs sm:placeholder:text-sm"
  />

  <Command.List data-search-dialog="">
    <Command.Empty class={cn("h-full py-0")}></Command.Empty>
    {@const currentRoute = $page.route.id}
    <Command.Group heading="Explore Data">
      {@const userAddresses = [
        $sepoliaStore?.address,
        $cosmosStore?.address
      ].filter(Boolean)}
      <Command.Item
        let:attrs
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
        <TableIcon class="mr-2 size-5" />
        <span>Your past transfers</span>
        {#if $page.route.id?.startsWith("/explorer/address")}
          <Badge
            variant="outline"
            class={cn(
              "px-2 py-1 m-0 ml-auto rounded-none text-xs",
              attrs["data-selected"]
                ? "text-black bg-union-accent"
                : "bg-primary-foreground"
            )}
          >
            active page
          </Badge>
        {/if}
      </Command.Item>
      <Command.Item
        let:attrs
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
        <BrainIcon class="mr-2 size-5" />
        <span>Live IBC transfer feed</span>
        {#if $page.route.id?.startsWith("/explorer/transfers")}
          <Badge
            variant="outline"
            class={cn(
              "px-2 py-1 m-0 ml-auto rounded-none text-xs",
              attrs["data-selected"]
                ? "text-black bg-union-accent"
                : "bg-primary-foreground"
            )}
          >
            active page
          </Badge>
        {/if}
      </Command.Item>
    </Command.Group>
    <Command.Separator />
    <Command.Group class="text-sm" heading="Interact with the network">
      <Command.Item
        let:attrs
        tabindex={3}
        class={cn(
          "hover:cursor-pointer",
          "focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1"
        )}
        onSelect={() => {
          goto(`/transfer`)
          commandDialogOpen = false
        }}
      >
        <DollarSignIcon class="mr-2 size-5" />
        <span>Execute cross chain transfers</span>
        {#if $page.route.id?.startsWith("/transfer")}
          <Badge
            variant="outline"
            class={cn(
              "px-2 py-1 m-0 ml-auto rounded-none text-xs",
              attrs["data-selected"]
                ? "text-black bg-union-accent"
                : "bg-primary-foreground"
            )}
          >
            active page
          </Badge>
        {/if}
      </Command.Item>
      <Command.Item
        let:attrs
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
        <SmileIcon class="mr-2 size-5" />
        <span>Get tokens from faucet</span>
        {#if $page.route.id?.startsWith("/faucet")}
          <Badge
            variant="outline"
            class={cn(
              "px-2 py-1 m-0 ml-auto rounded-none text-xs",
              attrs["data-selected"]
                ? "text-black bg-union-accent"
                : "bg-primary-foreground"
            )}
          >
            active page
          </Badge>
        {/if}
      </Command.Item>
    </Command.Group>
  </Command.List>
</Command.Dialog>

<style lang="postcss">
  :global(div[data-command-dialog-overlay], div[data-dialog-overlay]) {
    backdrop-filter: blur(5px);
  }

  :global([data-cmdk-group-heading]) {
    @apply text-gray-400;
  }

  /* :global(div[data-dialog-content]) {
    @apply mx-auto max-w-[450px];
  } */
</style>
