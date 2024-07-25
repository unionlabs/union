<script lang="ts">
import { formatUnits } from "viem"
import { cn } from "$lib/utilities/shadcn.ts"
import { truncate } from "$lib/utilities/format"
import * as Dialog from "$lib/components/ui/dialog"
import { Button } from "$lib/components/ui/button/index.js"
import type { Chain } from "$lib/types.ts"
import Precise from "$lib/components/precise.svelte"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import { showUnsupported } from "$lib/stores/user.ts"
import { ScrollArea } from "$lib/components/ui/scroll-area"
import { type Address, parseUnits, toHex } from "viem"

/**
 * TODO: format the balance to a readable format - in order to do that properly, need:
 *  - the balance,
 *  - the decimals,
 *  - whether it's evm or cosmos:
 *    - if evm then `Number(formatUnits(balance, decimals)).toFixed(2)`, - the 2 can be a 4 if you want more precision
 *    - if cosmos then: TBD
 */

export let dialogOpen = false
export let chain: Chain
export let assets: Array<{
  address: string
  balance: bigint
  decimals?: number
  symbol: string
}>

export let onAssetSelect: (data: {address: string, symbol: string}) => void
</script>

<Dialog.Root
  bind:open={dialogOpen}
  closeOnEscape={true}
  closeOnOutsideClick={true}
  preventScroll={true}
>
  <Dialog.Content
    class="max-w-[90%] sm:max-w-[450px] max-h-[95%] px-0 pt-4 pb-2 flex flex-col items-start"
  >
    <Dialog.Header class="max-h-min h-8 p-2">
      <Dialog.Title class="px-2">Select Asset</Dialog.Title>
    </Dialog.Header>
    <div class="w-full overflow-scroll">
      <ul>
          {#each assets as asset, index}
            {@const supportedAsset = getSupportedAsset(chain, asset.address)}
            {#if $showUnsupported || supportedAsset}
              <li
                class={cn(
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            )}
              >
                <Button
                  variant="ghost"
                  class={cn('size-full px-4 py-2 w-full text-foreground rounded-none flex ')}
                  on:click={() => {
                onAssetSelect({address: asset.address, symbol : supportedAsset ? supportedAsset.display_symbol : asset.symbol})
                dialogOpen = false
              }}
                >
                  <div class="size-full flex flex-col items-start" class:opacity-30={!supportedAsset}>
                    {truncate(supportedAsset ? supportedAsset.display_symbol : asset.symbol, 6)}
                  </div>
                  <p class="mb-auto text-lg font-black" class:opacity-30={!supportedAsset}>
                    {formatUnits(asset.balance, supportedAsset?.decimals ?? 0)}
                  </p>
                </Button>
              </li>
            {/if}
          {/each}
      </ul>
    </div>
  </Dialog.Content>
</Dialog.Root>
