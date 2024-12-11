<script lang="ts">
import CopyUrlButton from "$lib/components/TransferFrom/components/CopyUrlButton.svelte"
import ResetButton from "$lib/components/TransferFrom/components/ResetButton.svelte"
import type { IntentStore } from "../transfer/intents.ts"
import type { ValidationStore } from "../transfer/validation.ts"
import type { ContextStore } from "../transfer/context.ts"

interface DebugProps {
  intents: IntentStore
  validation: ValidationStore
  context: ContextStore
}

export let intents: DebugProps["intents"]
export let validation: DebugProps["validation"]
export let context: DebugProps["context"]

const { userAddress, sourceChain, destinationChain, balances, assetInfo, chains } = context
</script>

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
        <p class="text-sm">{key}: "{value}"</p>
      {/each}
    </div>

    <div class="summary mb-4">
      <h3 class="text-red-500">Validation Errors:</h3>
      {#each Object.entries($validation) as [key, value]}
        <p class="text-sm">{key}: "{value}"</p>
      {/each}
    </div>

    <div class="summary mb-4">
      <h3 class="text-orange-500">User Addresses:</h3>
      {#if $userAddress}
        {#each Object.entries($userAddress) as [key, value]}
          <p class="text-sm">{key}: "{value?.canonical}"</p>
        {/each}
      {:else}
        <p class="text-sm">No user addresses available</p>
      {/if}
    </div>

    <div class="summary mb-4">
      <h3 class="text-yellow-500">Selected Asset Info:</h3>
      {#if $assetInfo}
        {#each Object.entries($assetInfo) as [key, value]}
          <p class="text-sm">
            {key}: "{typeof value === 'bigint' ? value.toString() : value}"
          </p>
        {/each}
      {:else}
        <p class="text-sm">No asset selected</p>
      {/if}
    </div>

    <div class="summary mb-4">
      <h3 class="text-purple-500">Balances:</h3>
      {#if $balances.length}
        {#each $balances as balance}
          <div class="ml-2 mb-2">
            {#each Object.entries(balance) as [key, value]}
              <p class="text-sm">
                {key}: "{typeof value === 'bigint' ? value.toString() : value}"
              </p>
            {/each}
          </div>
        {/each}
      {:else}
        <p class="text-sm">No balances available</p>
      {/if}
    </div>

    <div class="summary mb-4">
      <h3 class="text-blue-500">Source Chain:</h3>
      {#if $sourceChain}
        {#each Object.entries($sourceChain) as [key, value]}
          <p class="text-sm">{key}: "{value}"</p>
        {/each}
      {:else}
        <p class="text-sm">No source chain selected</p>
      {/if}
    </div>

    <div class="summary mb-4">
      <h3 class="text-green-500">Destination Chain:</h3>
      {#if $destinationChain}
        {#each Object.entries($destinationChain) as [key, value]}
          <p class="text-sm">{key}: "{value}"</p>
        {/each}
      {:else}
        <p class="text-sm">No destination chain selected</p>
      {/if}
    </div>

    <div class="summary mb-4">
      <h3 class="text-indigo-500">Available Chains:</h3>
      {#if chains.length}
        {#each chains as chain}
          <div class="ml-2 mb-2">
            {#each Object.entries(chain) as [key, value]}
              <p class="text-sm">{key}: "{value}"</p>
            {/each}
          </div>
        {/each}
      {:else}
        <p class="text-sm">No chains available</p>
      {/if}
    </div>

  </div>
</div>