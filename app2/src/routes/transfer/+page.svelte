<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"
import Input from "$lib/components/ui/Input.svelte"

export const rawIntents = new RawIntentsStoreSvelte()

function resetAll() {
  rawIntents.clearUrlParameters()
  rawIntents.set({
    source: "",
    destination: "",
    asset: "",
    receiver: "",
    amount: ""
  })
}
</script>

<div class="max-w-3xl mx-auto p-6 bg-zinc-950 text-zinc-200 min-h-screen">
  <h1 class="text-2xl font-bold mb-6 text-sky-400">rawIntents Test Page (Svelte 5)</h1>
  
  <div class="mb-6 p-4 bg-zinc-800 rounded-lg border border-zinc-700">
    <h3 class="text-lg font-semibold mb-2 text-sky-300">Current State:</h3>
    <pre class="text-sm text-zinc-300 whitespace-pre-wrap break-all">{JSON.stringify({
      source: rawIntents.source,
      destination: rawIntents.destination,
      asset: rawIntents.asset,
      receiver: rawIntents.receiver,
      amount: rawIntents.amount
    }, null, 2)}</pre>
  </div>

  <div class="mb-6">
    <h2 class="text-xl font-semibold mb-4 text-sky-400">Test Form</h2>

    <Input
      id="source"
      label="Source"
      value={rawIntents.source}
      oninput={(e) => rawIntents.updateField('source', e)}
    />

    <Input
      id="destination"
      label="Destination" 
      value={rawIntents.destination}
      oninput={(e) => rawIntents.updateField('destination', e)}
    />

    <Input
      id="asset"
      label="Asset"
      value={rawIntents.asset}
      oninput={(e) => rawIntents.updateField('asset', e)}
    />

    <Input
      id="receiver"
      label="Receiver"
      value={rawIntents.receiver}
      oninput={(e) => rawIntents.updateField('receiver', e)}
    />

    <Input
      id="amount"
      label="Amount"
      value={rawIntents.amount}
      oninput={(e) => rawIntents.updateField('amount', e)}
    />
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
