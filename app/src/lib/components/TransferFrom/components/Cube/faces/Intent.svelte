<script lang="ts">
import Direction from "$lib/components/TransferFrom/components/Direction.svelte"
import SelectedAsset from "$lib/components/TransferFrom/components/SelectedAsset.svelte"
import type { Readable } from "svelte/store"
import type { ValidationStore } from "$lib/components/TransferFrom/transfer/validation.ts"
import { Button } from "$lib/components/ui/button"
import type { IntentsStore } from "$lib/components/TransferFrom/transfer/intents.ts"
import type { CubeFaces } from "$lib/components/TransferFrom/components/Cube/types.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { Input } from "$lib/components/ui/input"
import LoadingDots from "$lib/components/loading-dots.svelte"
import Token from "$lib/components/token.svelte"
import type { Chain, Ucs03Channel } from "$lib/types"
import ArrowRightIcon from "virtual:icons/lucide/arrow-right"
import { toDisplayName } from "$lib/utilities/chains"
import Address from "$lib/components/address.svelte"

interface Props {
  stores: {
    rawIntents: RawIntentsStore
    intents: Readable<IntentsStore>
    validation: Readable<ValidationStore>
  }
  rotateTo: (face: CubeFaces) => void
}

export let stores: Props["stores"]
export let rotateTo: Props["rotateTo"]
export let chains: Array<Chain>
export let channel: Readable<Ucs03Channel | null>
export let transferArgs: {
  baseToken: string
  baseAmount: bigint
  quoteToken: string
  quoteAmount: bigint
  receiver: string
  sourceChannelId: number
  ucs03address: string
} | null

let { rawIntents, intents, validation } = stores
</script>

<div class="flex flex-col w-full h-full ">

  <div class="text-primary p-2 flex items-center justify-between border-b-2">
    <span class="font-bold uppercase">Transfer</span>
  </div>
  <div class="flex flex-col h-full w-full justify-between p-4">
      <div class="flex flex-col gap-2">
      <Direction {intents} {validation} {rawIntents} getSourceChain={() => rotateTo("sourceFace")}
                 getDestinationChain={() => rotateTo("destinationFace")}/>
      <SelectedAsset {intents} {validation} {rawIntents} onSelectAsset={() => rotateTo("assetsFace")}/>
      <div class="flex flex-col gap-1 items-start">
        <Input
                id="amount"
                type="number"
                name="amount"
                minlength={1}
                maxlength={64}
                required={true}
                disabled={!$rawIntents.asset}
                autocorrect="off"
                placeholder="0.00"
                spellcheck="false"
                autocomplete="off"
                inputmode="decimal"
                data-field="amount"
                autocapitalize="none"
                pattern="^[0-9]*[.,]?[0-9]*$"
                class="p-1 {$validation.errors.amount ? 'border-red-500' : ''}"
                value={$intents.amount}
                on:input={event => rawIntents.updateField('amount', event)}
        />
        {#if $validation.errors.amount}
          <span class="text-red-500 text-sm">{$validation.errors.amount}</span>
        {/if}
        <Input
                type="text"
                id="receiver"
                name="receiver"
                required={true}
                disabled={!$intents.destinationChain}
                autocorrect="off"
                spellcheck="false"
                autocomplete="off"
                data-field="receiver"
                class="p-1 disabled:bg-black/30 {$validation.errors.receiver ? 'border-red-500' : ''}"
                placeholder="Enter destination address"
                value={$intents.receiver}
                on:input={event => rawIntents.updateField('receiver', event)}
        />
        {#if $validation.errors.receiver}
          <span class="text-red-500 text-sm">{$validation.errors.receiver}</span>
        {/if}
      </div>
      </div>

      {#if !$channel}
      <div>No recommended UCS03 channel to go from {toDisplayName($rawIntents.source, chains)} to {toDisplayName($rawIntents.destination, chains)}</div>
      {:else}
        <div class="flex flex-col gap-1 justify-end items-center">
          <div class="flex gap-4 text-muted-foreground text-xs">{$channel?.source_connection_id} | {$channel?.source_channel_id} <ArrowRightIcon />{$channel?.destination_connection_id} | {$channel?.destination_channel_id}</div> 
          {#if !$rawIntents.asset}
            Select an asset
          {:else}
            {#if !transferArgs}
              <LoadingDots/>
            {:else}
              <div class="flex-1 flex flex-col items-center text-xs">
                <Token amount={$rawIntents.amount} chainId={$rawIntents.destination} denom={transferArgs.quoteToken} {chains}/>
              </div>
              {#if $validation.isValid}
                <Address address={transferArgs.receiver} {chains} chainId={$channel.destination_chain_id}/>
              {/if}
              <Button
                      class="w-full mt-2"
                      disabled={!$validation.isValid}
                      on:click={() => rotateTo("verifyFace")}>Transfer
              </Button>
            {/if}
        {/if}
      </div>
    {/if}
  </div>
</div>
