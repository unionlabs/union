<!-- src/routes/+page.svelte -->
<script lang="ts">
import { Effect, Schema } from "effect"
import type { RpcType } from "$lib/schema/chain"
import {
  AptosTransfer,
  CosmosTransfer,
  EVMTransfer,
  type Transfer
} from "$lib/schema/transfer-arguments"
import { examples } from "$lib/examples/transfer-arguments"

type RpcTypeValue = Schema.Schema.Type<typeof RpcType>

let results: Array<{ type: string; data?: Transfer | undefined; error?: string | undefined }> = []

function handleResult(type: RpcTypeValue, parsed: Transfer | undefined, error: unknown) {
  if (parsed) {
    console.log(`${type} example:`, parsed)
    results = [...results, { type, data: parsed, error: undefined }]
  } else {
    console.error(`${type} example failed:`, error)
    results = [...results, { type, error: String(error), data: undefined }]
  }
}

function validateEvm() {
  const effect = Schema.decode(EVMTransfer)(examples.evm)
  Effect.runPromise(effect)
    .then(parsed => handleResult("evm", parsed, undefined))
    .catch(error => handleResult("evm", undefined, error))
}

function validateCosmos() {
  const effect = Schema.decode(CosmosTransfer)(examples.cosmos)
  Effect.runPromise(effect)
    .then(parsed => handleResult("cosmos", parsed, undefined))
    .catch(error => handleResult("cosmos", undefined, error))
}

function validateAptos() {
  const effect = Schema.decode(AptosTransfer)(examples.aptos)
  Effect.runPromise(effect)
    .then(parsed => handleResult("aptos", parsed, undefined))
    .catch(error => handleResult("aptos", undefined, error))
}
</script>

<main class="p-8 max-w-5xl mx-auto bg-zinc-950 min-h-screen">
  <h1 class="text-4xl font-bold text-zinc-100 mb-4 text-center">Transfer Schema Validation</h1>
  <p class="text-lg text-zinc-400 mb-8 text-center">Check the console for detailed logs of the examples.</p>

  <div class="grid gap-6 mb-8">
    {#each results as result}
      <div class="bg-zinc-800 rounded-lg p-6 shadow-md border border-zinc-700">
        <h2 class="text-2xl font-semibold text-zinc-200 mb-4">{result.type.toUpperCase()} Example</h2>
        {#if result.data}
          <p class="text-green-400 font-bold mb-4">Success!</p>
          <pre class="bg-zinc-900 text-zinc-700 p-4 rounded-md text-sm font-mono overflow-x-auto whitespace-pre-wrap border border-zinc-600">{JSON.stringify(result.data, (_, v) => typeof v === "bigint" ? v.toString() : v, 2)}</pre>
        {:else if result.error}
          <p class="text-red-400 font-bold mb-4 break-words">Error: {result.error}</p>
        {/if}
      </div>
    {/each}
  </div>

  <div class="flex justify-center gap-4 flex-wrap">
    <button on:click={validateEvm} class="bg-zinc-700 text-zinc-100 px-6 py-3 rounded-md text-base font-medium hover:bg-zinc-600 active:bg-zinc-800 transition duration-200 hover:-translate-y-1 active:translate-y-0">Validate EVM Example</button>
    <button on:click={validateCosmos} class="bg-zinc-700 text-zinc-100 px-6 py-3 rounded-md text-base font-medium hover:bg-zinc-600 active:bg-zinc-800 transition duration-200 hover:-translate-y-1 active:translate-y-0">Validate Cosmos Example</button>
    <button on:click={validateAptos} class="bg-zinc-700 text-zinc-100 px-6 py-3 rounded-md text-base font-medium hover:bg-zinc-600 active:bg-zinc-800 transition duration-200 hover:-translate-y-1 active:translate-y-0">Validate Aptos Example</button>
  </div>
</main>