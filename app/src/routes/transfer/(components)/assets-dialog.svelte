<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import type { Asset } from "$lib/wallet/types"
import { truncate } from "$lib/utilities/format"
import * as Dialog from "$lib/components/ui/dialog"
import { Button } from "$lib/components/ui/button/index.js"

export let dialogOpen = false
export let assets: Array<{
  address: string
  balance: bigint
  decimals: number
  symbol: string
}>
export let onAssetSelect: (asset: string) => void
</script>

<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  bind:open={dialogOpen}
  closeOnOutsideClick={true}
>
  <Dialog.Content
    class="max-w-[90%] sm:max-w-[450px]  overflow-auto px-0 pt-4 pb-2 flex flex-col items-start"
  >
    <Dialog.Header class="max-h-min h-8 p-2">
      <Dialog.Title class="px-2">Select Asset</Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <ul class="">
        {#each assets as { address, symbol, decimals, balance }, index}
          <li
            class={cn(
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            )}
          >
            <Button
              variant="ghost"
              class={cn(
                'size-full px-4 py-2 w-full text-foreground rounded-none flex ',
              )}
              on:click={() => {onAssetSelect(symbol); dialogOpen = false }}
            >
              <div class="size-full flex flex-col items-start">
                {truncate(symbol, 12)}
              </div>
              <p class="mb-auto text-lg font-black">{balance}</p>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
