<script lang="ts">
import Direction from "$lib/components/TransferFrom/components/Direction.svelte"
import SelectedAsset from "$lib/components/TransferFrom/components/SelectedAsset.svelte"
import { Button } from "$lib/components/ui/button"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { Input } from "$lib/components/ui/input"
import LoadingDots from "$lib/components/loading-dots.svelte"
import Token from "$lib/components/token.svelte"
import ArrowRightIcon from "virtual:icons/lucide/arrow-right"
import { toDisplayName } from "$lib/utilities/chains"
import Address from "$lib/components/address.svelte"
import type { Intents, TransferArgs } from "$lib/components/TransferFrom/transfer/types.ts"
import type { Chain } from "$lib/types.ts"
import type { Readable } from "svelte/store"

interface Props {
  rawIntents: RawIntentsStore
  intents: Intents
  validation: any
  chains: Array<Chain>
  rotateTo: (face: CubeFaces) => void
}

export let rawIntents: Props["rawIntents"]
export let intents: Props["intents"]
export let validation: Props["validation"]
export let chains: Props["chains"]
export let rotateTo: Props["rotateTo"]
</script>

<div class="flex flex-col w-full h-full ">

  <div class="text-primary p-2 flex items-center justify-between border-b-2">
    <span class="font-bold uppercase">Transfer</span>
  </div>
  <div class="flex flex-col h-full w-full justify-between p-4">
    <div class="flex flex-col gap-2">
      <Direction {rawIntents} {intents} {validation} getSourceChain={() => rotateTo("sourceFace")}
                 getDestinationChain={() => rotateTo("destinationFace")}/>
      <SelectedAsset {intents} {validation} {rawIntents} {chains} onSelectAsset={() => rotateTo("assetsFace")}/>
      <div class="flex flex-col gap-1 items-start">
        <Input
                id="amount"
                type="text"
                name="amount"
                required={true}
                disabled={!$rawIntents.asset}
                autocorrect="off"
                placeholder="0.00"
                spellcheck="false"
                autocomplete="off"
                inputmode="decimal"
                data-field="amount"
                autocapitalize="none"
                pattern="^[0-9]*[.]?[0-9]*$"
                class="p-1 {validation.errors.amount ? 'border-red-500' : ''}"
                value={$rawIntents.amount}
                on:input={(event) => {
                      const input = event.currentTarget;
                      const value = input.value;
                      // Only allow numbers and a single decimal point
                      if (value === '' || /^\d*\.?\d*$/.test(value)) {
                        rawIntents.updateField('amount', event);
                      } else {
                        // If invalid input, revert to previous valid value
                        input.value = $rawIntents.amount;
                      }
                    }}
        />
        {#if validation.errors.amount}
          <span class="text-red-500 text-sm">{validation.errors.amount}</span>
        {/if}
        <Input
                type="text"
                id="receiver"
                name="receiver"
                required={true}
                disabled={!intents.destinationChain}
                autocorrect="off"
                spellcheck="false"
                autocomplete="off"
                data-field="receiver"
                class="p-1 disabled:bg-black/30 {validation.errors.receiver ? 'border-red-500' : ''}"
                placeholder="Enter destination address"
                value={$rawIntents.receiver}
                on:input={event => rawIntents.updateField('receiver', event)}
        />
        {#if $rawIntents.receiver === intents.ownWallet}
          <button class="text-xs text-muted-foreground" on:click={() => rawIntents.updateField("receiver", "")}>Reset
          </button>
        {:else}
          <button class="text-xs text-muted-foreground"
                  on:click={() => rawIntents.updateField('receiver', intents.ownWallet)}>Use connected wallet
          </button>
        {/if}
        {#if validation.errors.receiver}
          <span class="text-red-500 text-sm">{validation.errors.receiver}</span>
        {/if}
      </div>
    </div>

    {#if !intents.channel}
      <div>No recommended UCS03 channel to go from {toDisplayName($rawIntents.source, chains)}
        to {toDisplayName($rawIntents.destination, chains)}</div>
    {:else}
      <div class="flex flex-col gap-1 justify-end items-center">
        <div class="flex gap-4 text-muted-foreground text-xs">{intents.channel.source_connection_id}
          | {intents.channel.source_channel_id}
          <ArrowRightIcon/>{intents.channel.destination_connection_id} | {intents.channel.destination_channel_id}
        </div>
        {#if !$rawIntents.asset}
          <p class="text-xs">Select an asset</p>
        {:else if !$rawIntents.source || !$rawIntents.destination}
          <p class="text-xs">Select source and destination</p>
        {:else if validation.args === "NO_QUOTE_AVAILABLE"}
          <div class="text-xs text-center">No Quote Token available for this transfer. Sending new assets to Cosmos is
            currently not supported and will be enabled in an update soon.
          </div>
        {:else if intents.quoteToken}
          <div class="flex-1 flex flex-col items-center text-xs">
            <Token chainId={$rawIntents.destination} denom={intents.quoteToken} {chains}/>
          </div>
          {#if validation.isValid}
            <Address address={intents.receiver} {chains} chainId={intents.channel.destination_chain_id}/>
          {/if}
        {/if}
        <Button class="w-full mt-2" disabled={!validation.isValid} on:click={() => rotateTo("verifyFace")}>Transfer
        </Button>
      </div>
    {/if}
  </div>
</div>
