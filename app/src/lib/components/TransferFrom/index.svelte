<script lang="ts">
  import {TRANSFER_DEBUG} from "$lib/components/TransferFrom/config.ts"
  import CopyUrlButton from "$lib/components/TransferFrom/components/CopyUrlButton.svelte"
  import ResetButton from "$lib/components/TransferFrom/components/ResetButton.svelte"
  import {createTransferStore} from "$lib/components/TransferFrom/transfer.ts"
  import DebugBox from "$lib/components/TransferFrom/components/DebugBox.svelte"

  const {intents, chains, userAddress, sourceChain, destinationChain, balances, assetInfo} =
    createTransferStore()
</script>

<form
        id="transfer"
        name="transfer"
        action="transfer"
        data-form="transfer"
        class="flex flex-col p-4"
>
  <div class="flex flex-col gap-4">
    <div class="flex flex-col gap-1">
      <label for="source" class="text-sm font-medium">Source</label>
      <input
              type="text"
              id="source"
              name="source"
              placeholder="Enter source chain"
              class="w-[300px] p-1 {$intents.errors.source ? 'border-red-500' : ''}"
              value={$intents.source}
              on:input={event => intents.updateField('source', event)}
      />
      {#if $intents.errors.source}
        <span class="text-red-500 text-sm">{$intents.errors.source}</span>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      <label for="destination" class="text-sm font-medium">Destination</label>
      <input
              type="text"
              id="destination"
              name="destination"
              placeholder="Enter destination chain"
              class="w-[300px] p-1 {$intents.errors.destination ? 'border-red-500' : ''}"
              value={$intents.destination}
              on:input={event => intents.updateField('destination', event)}
      />
      {#if $intents.errors.destination}
        <span class="text-red-500 text-sm">{$intents.errors.destination}</span>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      <label for="asset" class="text-sm font-medium">Asset</label>
      <input
              type="text"
              id="asset"
              name="asset"
              placeholder="Enter asset"
              class="w-[300px] p-1 {$intents.errors.asset ? 'border-red-500' : ''}"
              value={$intents.asset}
              on:input={event => intents.updateField('asset', event)}
      />
      {#if $intents.errors.asset}
        <span class="text-red-500 text-sm">{$intents.errors.asset}</span>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      <label for="amount" class="text-sm font-medium">Amount</label>
      <input
              id="amount"
              type="number"
              name="amount"
              minlength={1}
              maxlength={64}
              required={true}
              disabled={false}
              autocorrect="off"
              placeholder="0.00"
              spellcheck="false"
              autocomplete="off"
              inputmode="decimal"
              data-field="amount"
              autocapitalize="none"
              pattern="^[0-9]*[.,]?[0-9]*$"
              class="w-[300px] p-1 {$intents.errors.amount ? 'border-red-500' : ''}"
              value={$intents.amount}
              on:input={event => intents.updateField('amount', event)}
      />
      {#if $intents.errors.amount}
        <span class="text-red-500 text-sm">{$intents.errors.amount}</span>
      {/if}
    </div>

    <div class="flex flex-col gap-1">
      <label for="receiver" class="text-sm font-medium">Receiver Address</label>
      <input
              type="text"
              id="receiver"
              name="receiver"
              required={true}
              disabled={false}
              autocorrect="off"
              spellcheck="false"
              autocomplete="off"
              data-field="receiver"
              class="w-[300px] p-1 disabled:bg-black/30 {$intents.errors.receiver ? 'border-red-500' : ''}"
              placeholder="Enter destination address"
              value={$intents.receiver}
              on:input={event => intents.updateField('receiver', event)}
      />
      {#if $intents.errors.receiver}
        <span class="text-red-500 text-sm">{$intents.errors.receiver}</span>
      {/if}
    </div>
  </div>
</form>

{#if TRANSFER_DEBUG}
  <DebugBox {intents} {chains} {userAddress} {sourceChain} {destinationChain} {balances} {assetInfo}/>
{/if}