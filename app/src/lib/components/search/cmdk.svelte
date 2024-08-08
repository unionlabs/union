<script lang="ts">
import {
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  isValidCosmosTxHash,
  isValidBech32Address
} from "@union/client"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { goto } from "$app/navigation"
import { cn } from "$lib/utilities/shadcn"
import SearchBar from "./search-bar.svelte"
import Kbd from "$lib/components/kbd.svelte"
import { sepoliaStore } from "$lib/wallet/evm"
import { cosmosStore } from "$lib/wallet/cosmos"
import { derived, writable } from "svelte/store"
import SmileIcon from "virtual:icons/lucide/smile"
import TableIcon from "virtual:icons/lucide/table"
import BrainIcon from "virtual:icons/lucide/brain"
import Badge from "$lib/components/ui/badge/badge.svelte"
import * as Command from "$lib/components/ui/command/index.ts"
import DollarSignIcon from "virtual:icons/lucide/badge-dollar-sign"

let searchInput = writable("")
searchInput.update($searchInput => $searchInput.replaceAll(" ", ""))

let commandDialogOpen = false

let windowSize = { width: window.innerWidth, height: window.innerHeight }

function handleKeyDown(event: KeyboardEvent) {
  if (event.key !== "k" || !(event.metaKey || event.ctrlKey)) return
  event.preventDefault()
  commandDialogOpen = true
}

const handleResize = () =>
  requestAnimationFrame(() => {
    windowSize = { width: window.innerWidth, height: window.innerHeight }
  })

onMount(() => {
  window.addEventListener("resize", handleResize)
  document.addEventListener("keydown", handleKeyDown)
  return () => {
    window.removeEventListener("resize", handleResize)
    document.removeEventListener("keydown", handleKeyDown)
  }
})

function validTxHash(hash: string) {
  return isValidCosmosTxHash(hash) || isValidEvmTxHash(hash)
}

function validAddress(address: string) {
  return isValidBech32Address(address) || isValidEvmAddress(address)
}

function validateAndFilterMultiAddress(addressArray: Array<string>) {
  return addressArray.filter(validAddress)
}

let computedSearchInputResult = derived(searchInput, $searchInput => {
  if (validTxHash($searchInput)) {
    return {
      type: "tx",
      value: $searchInput,
      truncated: [
        truncateAddress({
          address: $searchInput,
          length: windowSize.width >= 500 ? 16 : 11
        })
      ]
    }
  }

  const input = validateAndFilterMultiAddress(
    $searchInput.includes("-") ? $searchInput.split("-") : [$searchInput]
  )

  if (input.length > 0) {
    return {
      type: "address",
      value: $searchInput,
      truncated: input.map(address =>
        truncateAddress({
          address,
          length: windowSize.width >= 500 ? 17 : 11
        })
      )
    }
  }

  return { type: "unknown", value: $searchInput }
})

function onEnterPress(event: KeyboardEvent) {
  event.stopPropagation()
  if (event.key === "Escape") commandDialogOpen = false
  if (event.key !== "Enter") return

  if ($computedSearchInputResult.type === "tx") {
    goto(`/explorer/transfers/${$searchInput}`)
    commandDialogOpen = false
    $searchInput = ""
  }

  if ($computedSearchInputResult.type === "address") {
    goto(`/explorer/address/${$searchInput}`)
    commandDialogOpen = false
    $searchInput = ""
  }
}

/**
 * the values in the combobox should not be tabbable
 * instead, arrow keys should be used to navigate and enter to select
 * @ref: https://www.w3.org/WAI/ARIA/apg/patterns/combobox
 */
const DISABLE_TAB_INDEX = -1
</script>

<SearchBar
  searchInput={$searchInput}
  windowWidth={windowSize.width}
  onInputClick={() => (commandDialogOpen = true)}
/>

<Command.Dialog
  preventScroll={true}
  closeOnEscape={true}
  label="Search Dialog"
  onKeydown={onEnterPress}
  closeOnOutsideClick={true}
  bind:open={commandDialogOpen}
  class={cn(
    ' w-full',
    'border-solid shadow-2xl',
  )}
>
  <Command.Root
    loop={true}
    shouldFilter={true}
    filter={(value, search) => (value.includes(search) ? 1 : 0)}

 class={cn('text-foreground bg-background')}
    
  >
    <Command.Input
      type="text"
      name="search"
      autofocus={true}
      autocorrect="off"
      inputmode="search"
      autocomplete="off"
      spellcheck="false"
      autocapitalize="off"
      pattern="[A-Za-z0-9\-]+"
      bind:value={$searchInput}
      placeholder="Navigate, search for address or tx by hash..."
    />

    <Command.List data-search-dialog="">
      {@const shouldRenderTheBelow =
        $computedSearchInputResult.type === 'tx' || $computedSearchInputResult.type === 'address'}
      {#if shouldRenderTheBelow}
        <Command.Empty
          data-cmdk-empty
          autocorrect="off"
          spellcheck="false"
          autocapitalize="off"
          class={cn(
            'h-full px-2 sm:px-3 py-4 text-left flex justify-between text-black dark:text-inherit',
          )}
        >
          {#if $computedSearchInputResult.type === 'tx'}
            <span>
              {$computedSearchInputResult.truncated}
            </span>
          {:else if $computedSearchInputResult.type === 'address'}
            {@const truncatedAddresses = $computedSearchInputResult.truncated ?? []}
            <ul>
              {#each truncatedAddresses as address}
                <li>{address}</li>
              {/each}
            </ul>
          {/if}
          <div class="my-auto">
            <Kbd class="top-1">Enter</Kbd>
            <span class="font-normal"> to navigate</span>
          </div>
        </Command.Empty>
      {/if}


<Command.Group heading="Interact with the testnet" class={cn('text-black bg-background')}>

        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/faucet`)
            commandDialogOpen = false
          }}
        >
          <SmileIcon class="mr-2 size-5" />
          <span>Get tokens from faucet</span>
          {#if $page.route.id?.startsWith('/faucet')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
  
        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/transfer`)
            commandDialogOpen = false
          }}
        >
          <DollarSignIcon class="mr-2 size-5" />
          <span>Transfer assets across chains</span>
          {#if $page.route.id?.startsWith('/transfer')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
        
      </Command.Group>
      <Command.Separator />
      
      <Command.Group heading="Explore Data" class={cn('text-black bg-background')}>
        {@const userAddresses = [$sepoliaStore?.address, $cosmosStore?.address].filter(Boolean)}
        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            userAddresses && userAddresses.length === 0 ? 'hidden' : '',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={_value => {
            goto(`/explorer/address/${userAddresses.join('-')}`)
            commandDialogOpen = false
          }}
        >
          <TableIcon class="mr-2 size-5" />
          <span>Your transfers</span>
          {#if $page.route.id?.startsWith('/explorer/address')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/explorer/transfers`)
            commandDialogOpen = false
          }}
        >
          <BrainIcon class="mr-2 size-5" />
          <span>All transfers</span>
          {#if $page.route.id?.startsWith('/explorer/transfers')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/explorer/connections`)
            commandDialogOpen = false
          }}
        >
          <BrainIcon class="mr-2 size-5" />
          <span>IBC connections</span>
          {#if $page.route.id?.startsWith('/explorer/connections')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
        
        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/explorer/channels`)
            commandDialogOpen = false
          }}
        >
          <BrainIcon class="mr-2 size-5" />
          <span>IBC channels</span>
          {#if $page.route.id?.startsWith('/explorer/channels')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>

        <Command.Item
          let:attrs
          tabindex={DISABLE_TAB_INDEX}
          class={cn(
            'hover:cursor-pointer',
            'focus:ring-1 focus:ring-union-accent-300 focus:ring-opacity-75 focus:rounded-none my-1',
          )}
          onSelect={() => {
            goto(`/explorer/index-status`)
            commandDialogOpen = false
          }}
        >
          <BrainIcon class="mr-2 size-5" />
          <span>Hubble index status</span>
          {#if $page.route.id?.startsWith('/explorer/index-status')}
            <Badge
              variant="outline"
              class={cn(
                'px-2 py-1 m-0 ml-auto rounded-none text-xs',
                attrs['data-selected'] ? 'text-black bg-union-accent' : 'bg-primary-foreground',
              )}
            >
              active page
            </Badge>
          {/if}
        </Command.Item>
        
      </Command.Group>


      
    </Command.List>
  </Command.Root>
</Command.Dialog>

<style lang="postcss">
  :global(div[data-command-dialog-overlay], div[data-dialog-overlay]) {
    backdrop-filter: blur(5px);
  }

  :global([data-cmdk-group-heading]) {
    @apply text-muted-foreground;
  }
</style>
