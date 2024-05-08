<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import Search from "virtual:icons/lucide/search"
import * as Dialog from "$lib/components/ui/dialog"
import type { Asset } from "$lib/constants/assets.js"
import { Input } from "$lib/components/ui/input/index.js"
import { Button } from "$lib/components/ui/button/index.js"

export let dialogOpen = false
export let handleAssetSearch: (event: InputEvent) => void
export let handleAssetSelect: (asset: string) => void
export let assetSearchResults: Array<Asset>
</script>

<Dialog.Root
  closeOnEscape={true}
  preventScroll={true}
  bind:open={dialogOpen}
  closeOnOutsideClick={true}
>
  <Dialog.Content
    class="max-w-[90%] sm:max-w-[400px] border-[1px] border-solid border-accent overflow-auto px-0 pt-3 pb-0 flex flex-col items-start rounded-md"
  >
    <Dialog.Header class="max-h-min h-8 p-2">
      <Dialog.Title class="font-extrabold text-2xl pl-3 -mt-2">Select a token</Dialog.Title>
    </Dialog.Header>
    <Dialog.Description class="size-full">
      <div class="relative mr-auto flex-1 w-full">
        <Search class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          type="search"
          pattern="[a-z]"
          autocorrect="off"
          spellcheck="false"
          placeholder="OSMO…"
          autocapitalize="off"
          on:input={handleAssetSearch}
          class="w-full rounded-none bg-current/95 pl-8 self-stretch lowercase border-x-0 focus-visible:ring-0"
        />
      </div>
      <ul class="my-3 mx-2 space-y-1">
        {#each assetSearchResults as { denom, symbol, explorerLink }, index}
          <li
            class={cn([
              // live ? 'cursor-pointer' : 'cursor-not-allowed',
              'pb-2 dark:text-accent-foreground flex flex-col h-full justify-start align-middle space-x-3.5',
            ])}
          >
            <Button
              variant="ghost"
              class={cn([
                'w-full flex justify-start space-x-4 p-2 rounded-none pl-3 h-[55px] my-auto',
              ])}
              on:click={() => handleAssetSelect('osmo-test-1')}
            >
              <img
                alt={`osmosis logo`}
                class="size-10 my-auto mr-auto"
                src={'/images/icons/osmosis.svg'}
              />
              <div class="size-full mr-auto flex flex-col items-start">
                <span
                  class="my-auto text-[22px] font-extrabold mr-auto w-full text-left justify-between text-foreground"
                >
                  {symbol}
                </span>
                <span class="text-xs text-muted-foreground">osmo-test-5</span>
              </div>
              <p class="mb-auto text-lg font-black">420.69</p>
            </Button>
          </li>
        {/each}
      </ul>
    </Dialog.Description>
  </Dialog.Content>
</Dialog.Root>
