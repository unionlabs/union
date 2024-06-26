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
import { toast } from "svelte-sonner"

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

export let onAssetSelect: (asset: string) => void

const copyAddress = (asset: { address: string }) => {
  if (asset.address) {
    navigator.clipboard
      .writeText(asset.address)
      .then(() => toast.info("Address copied!"))
      .catch(err => toast.error("Failed to copy address"))
  }
}
</script>

<Dialog.Root
  bind:open={dialogOpen}
  closeOnEscape={true}
  closeOnOutsideClick={true}
  preventScroll={true}
>
  <Dialog.Content
    class="max-w-[90%] sm:max-w-[450px]  overflow-auto px-0 pt-4 pb-2 flex flex-col items-start"
  >
    <Dialog.Header class="max-h-min h-8 p-2">
      <Dialog.Title class="px-2">Select Asset</Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <ul class="">
        {#each assets as asset, index}
          {@const supportedAsset = getSupportedAsset(chain, asset.address)}
          {#if $showUnsupported || supportedAsset}
            <li
              class={cn(
              'pb-2 dark:text-accent-foreground flex h-full justify-start ali',
            )}
            >
              <button type="button" on:click={() => copyAddress(asset)} variant="ghost" class="group pr-2 pl-4">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="size-4 opacity-50 group-hover:fill-accent group-hover:opacity-100">
                  <path d="M7 3.5A1.5 1.5 0 0 1 8.5 2h3.879a1.5 1.5 0 0 1 1.06.44l3.122 3.12A1.5 1.5 0 0 1 17 6.622V12.5a1.5 1.5 0 0 1-1.5 1.5h-1v-3.379a3 3 0 0 0-.879-2.121L10.5 5.379A3 3 0 0 0 8.379 4.5H7v-1Z" />
                  <path d="M4.5 6A1.5 1.5 0 0 0 3 7.5v9A1.5 1.5 0 0 0 4.5 18h7a1.5 1.5 0 0 0 1.5-1.5v-5.879a1.5 1.5 0 0 0-.44-1.06L9.44 6.439A1.5 1.5 0 0 0 8.378 6H4.5Z" />
                </svg>
              </button>
              <Button
                variant="ghost"
                class={cn('size-full px-2 py-2 w-full text-foreground rounded-none flex justify-between pr-4')}
                on:click={() => {
                onAssetSelect(asset.symbol)
                dialogOpen = false
              }}
              >
                <div class="flex flex-col items-start" class:opacity-30={!supportedAsset}>
                  {truncate(supportedAsset ? supportedAsset.display_symbol : asset.symbol, 6)}
                </div>
                <p class="text-lg font-black" class:opacity-30={!supportedAsset}>
                  <Precise {chain} {asset} showToolTip/>
                </p>
              </Button>
            </li>
          {/if}
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
