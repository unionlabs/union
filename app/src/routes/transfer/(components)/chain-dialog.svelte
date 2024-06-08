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
      'border-solid border-accent overflow-auto flex flex-col items-start rounded-md',
      'max-w-[90%] sm:max-w-[375px] border-[1px] pt-4 pb-1 px-0',
    )}
  >
    <Dialog.Header class="max-h-min p-2 w-full space-y-3">
      <Dialog.Title class="font-extrabold text-2xl text-center -mt-2">
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
              variant={selectedChain === chain.chain_id ? 'secondary' : 'ghost'}
              on:click={() => {onChainSelect(chain.chain_id); dialogOpen = false}}
              class={cn('size-full px-4 py-2 w-full rounded-none flex flex-col items-start')}
            >
                <div
                  class="text-lg font-bold"
                >
                  {chain.display_name}
                </div>
                <div class="text-xs text-muted-foreground">{chain.chain_id}</div>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
