<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"

export const RawIntents = new RawIntentsStoreSvelte()

function resetAll() {
  RawIntents.clearUrlParameters()
  RawIntents.set({
    source: "",
    destination: "",
    asset: "",
    receiver: "",
    amount: ""
  })
}
</script>

<div class="max-w-3xl mx-auto p-6 bg-zinc-900 text-zinc-200 min-h-screen">
  <h1 class="text-2xl font-bold mb-6 text-sky-400">RawIntents Test Page (Svelte 5)</h1>
  
  <div class="mb-6 p-4 bg-zinc-800 rounded-lg border border-zinc-700">
    <h3 class="text-lg font-semibold mb-2 text-sky-300">Current State:</h3>
    <pre class="text-sm text-zinc-300 whitespace-pre-wrap break-all">{JSON.stringify({
      source: RawIntents.source,
      destination: RawIntents.destination,
      asset: RawIntents.asset,
      receiver: RawIntents.receiver,
      amount: RawIntents.amount
    }, null, 2)}</pre>
  </div>

  <div class="mb-6">
    <h2 class="text-xl font-semibold mb-4 text-sky-400">Test Form</h2>

    <div class="mb-4">
      <label for="source" class="block mb-1 font-medium text-zinc-300">Source:</label>
      <input
              id="source"
              type="text"
              value={RawIntents.source}
              oninput={(e) => RawIntents.updateField('source', e)}
              class="w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200 focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
    </div>

    <div class="mb-4">
      <label for="destination" class="block mb-1 font-medium text-zinc-300">Destination:</label>
      <input
              id="destination"
              type="text"
              value={RawIntents.destination}
              oninput={(e) => RawIntents.updateField('destination', e)}
              class="w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200 focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
    </div>

    <div class="mb-4">
      <label for="asset" class="block mb-1 font-medium text-zinc-300">Asset:</label>
      <input
              id="asset"
              type="text"
              value={RawIntents.asset}
              oninput={(e) => RawIntents.updateField('asset', e)}
              class="w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200 focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
    </div>

    <div class="mb-4">
      <label for="receiver" class="block mb-1 font-medium text-zinc-300">Receiver:</label>
      <input
              id="receiver"
              type="text"
              value={RawIntents.receiver}
              oninput={(e) => RawIntents.updateField('receiver', e)}
              class="w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200 focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
    </div>

    <div class="mb-4">
      <label for="amount" class="block mb-1 font-medium text-zinc-300">Amount:</label>
      <input
              id="amount"
              type="text"
              value={RawIntents.amount}
              oninput={(e) => RawIntents.updateField('amount', e)}
              class="w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200 focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
    </div>
  </div>

  <div class="mt-6">
    <button
            onclick={resetAll}
            class="mr-2 mb-2 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
    >
      Reset All
    </button>
  </div>
</div>