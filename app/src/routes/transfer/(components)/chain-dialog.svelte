<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import * as Dialog from "$lib/components/ui/dialog"
import { Button } from "$lib/components/ui/button/index.js"
import type { LooseAutocomplete } from "$lib/utilities/types.ts"

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
