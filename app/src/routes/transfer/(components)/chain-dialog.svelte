<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import * as Dialog from "$lib/components/ui/dialog"
import { Input } from "$lib/components/ui/input/index.js"
import { Button } from "$lib/components/ui/button/index.js"
import type { LooseAutocomplete } from "$lib/utilities/types.ts"
import type { Writable } from "svelte/store"

export let kind: "from" | "to"
export let dialogOpen = false
export let onChainSelect: (newSelectedChain: string) => void
export let chains: Array<{ chain_id: string; display_name: string }>
export let selectedChain: string

$: document.body.style.overflow = dialogOpen ? "hidden" : "auto"
</script>

<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  bind:open={dialogOpen}
  closeOnOutsideClick={true}
>
  <Dialog.Content
    class={cn(
      'border-solid overflow-auto flex flex-col items-start p-0 pt-4 pb-2',
    )}
  >
    <Dialog.Header class="max-h-min p-2 w-full">
      <Dialog.Title class="px-2">
        Select {kind} Network
      </Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <!--
      <div class="relative mr-auto flex-1 w-full">
        <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          type="search"
          pattern="[a-z]"
          autocorrect="off"
          spellcheck="false"
          autocapitalize="off"
          placeholder="choose origin chain"
          on:input={event => handleChainSearch(event)}
          class="w-full bg-current/95 pl-8 self-stretch lowercase focus-visible:ring-0 rounded-none focus-visible:outline-none border-x-0"
        />
      </div>
      !-->
      <ul class="flex flex-col">
        {#each chains as chain, index}
          <li
            class={cn(
              'dark:text-accent-foreground flex flex-col',
            )}
          >
            <Button
              variant={'ghost'}
              on:click={() => {onChainSelect(chain.chain_id); dialogOpen = false}}
              class={cn('size-full px-4 py-2 w-full text-foreground rounded-none flex flex-col items-start',
                selectedChain === chain.chain_id ? 'bg-foreground text-background' : '' 
              
              )}
            >
                <div
                  class="text-lg font-bold"
                >
                  {chain.display_name}
                </div>
                <div class="text-xs -mt-1">{chain.chain_id}</div>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
