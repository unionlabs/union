<script lang="ts">
import { createIntentStore } from "./intents.ts"
import { debounce } from "$lib/utilities"
import type { Chain } from "$lib/types.ts"
import { TRANSFER_DEBUG } from "$lib/components/TransferFrom/config.ts"
import CopyUrlButton from "$lib/components/TransferFrom/components/CopyUrlButton.svelte"
import { Button } from "$lib/components/ui/button"
import ResetButton from "$lib/components/TransferFrom/components/ResetButton.svelte"

export let chains: Array<Chain>
$: console.log(chains)

const intents = createIntentStore()

async function handleSubmit(event) {
  event.preventDefault()
  event.stopPropagation()
}
</script>

<form
        id="transfer"
        name="transfer"
        action="transfer"
        data-form="transfer"
        class="flex flex-col p-4"
        on:submit={handleSubmit}
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
              on:input={event =>
                        debounce(
                            () => intents.updateField('source', event.target?.value || ''),
                            1_000
                        )()
                    }
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
              on:input={event =>
                        debounce(
                            () => intents.updateField('destination', event.target?.value || ''),
                            1_000
                        )()
                    }
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
              on:input={event =>
                        debounce(
                            () => intents.updateField('asset', event.target?.value || ''),
                            1_000
                        )()
                    }
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
              on:input={event =>
                        debounce(
                            () => intents.updateField('amount', event.target?.value || ''),
                            1_000
                        )()
                    }
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
              on:input={event =>
                        debounce(
                            () => intents.updateField('receiver', event.target?.value || ''),
                            1_000
                        )()
                    }
      />
      {#if $intents.errors.receiver}
        <span class="text-red-500 text-sm">{$intents.errors.receiver}</span>
      {/if}
    </div>
  </div>
</form>

{#if TRANSFER_DEBUG}
  <div class="p-4 w-full">
    <div class="p-4 bg-black w-full">

      <div class="mb-4 flex items-center gap-4">
        <CopyUrlButton/>
        <ResetButton onReset={intents.reset}/>
      </div>
      
      <h2 class="mb-4">TRANSFER DEBUG</h2>

      <div class="summary mb-4">
        <h3 class="text-union-accent-500">Raw Intents</h3>
        {#each Object.entries($intents) as [key, value]}
          {#if key !== "errors" && key !== "isValid"}
            <p class="text-sm">{key}: "{value}"</p>
          {/if}
        {/each}
      </div>

      <div class="summary mb-4">
        <h3 class="text-red-500">Errors:</h3>
        {#each Object.entries($intents.errors) as [key, value]}
          <p class="text-sm">{key}: "{value}"</p>
        {/each}
      </div>
    </div>
  </div>
{/if}