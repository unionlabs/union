<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { TransferSubmission, nextState, hasFailedExit, isComplete } from "$lib/services/transfer"
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"

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

/* Hack to be able to JSON.stringify BigInt */
interface BigInt {
  toJSON: () => string
}

BigInt["prototype"].toJSON = function () {
  return this.toString()
}

let transferState = $state<TransferSubmission>(TransferSubmission.Pending())

import { sepolia } from "viem/chains"
import type { TransactionParams } from "$lib/services/transfer/machine"
import SectionTitle from "$lib/components/ui/SectionTitle.svelte"

const transactionParams: TransactionParams = {
  chain: sepolia,
  account: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA" as const, // TODO: Get from wallet
  value: 1n,
  to: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA" as const // TODO: Get from form
}

async function submit() {
  const chainsData = Option.getOrNull(chains.data)
  if (!chainsData) return

  const sourceChain = chainsData.find(c => c.chain_id === transactionParams.chain.id.toString())
  if (!sourceChain) return

  transferState = await nextState(transferState, transactionParams, sourceChain)
  while (!hasFailedExit(transferState)) {
    transferState = await nextState(transferState, transactionParams, sourceChain)
    // If we're in the final state (TransferReceipt.Complete), stop
    if (isComplete(transferState)) {
      break
    }
  }
}
</script>

<Sections>
  <Card>
    <pre class="text-sm text-zinc-300 whitespace-pre-wrap break-all">{JSON.stringify({
      source: rawIntents.source,
      destination: rawIntents.destination,
      asset: rawIntents.asset,
      receiver: rawIntents.receiver,
      amount: rawIntents.amount
    }, null, 2)}</pre>
  </Card>

  <section class="flex flex-col gap-4">
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

    <div class="flex flex-col gap-4">
      <div class="flex gap-4">
        <Button 
          class="mt-4 self-start"
          variant="primary"
          onclick={submit}
          disabled={transferState._tag !== "Pending" && !hasFailedExit(transferState) && !isComplete(transferState)}
        >
          {#if transferState._tag !== "Pending" && !hasFailedExit(transferState) && !isComplete(transferState)}
            Submitting...
          {:else if hasFailedExit(transferState)}
            Retry
          {:else}
            Submit
          {/if}
        </Button>
        <Button 
          class="mt-4 self-start"
          variant="secondary"
          onclick={resetAll}
          disabled={transferState._tag !== "Pending" && !hasFailedExit(transferState)}
        >
          Reset All
        </Button>
      </div>
      {JSON.stringify(transferState, null, 2)}
    </div>
  </section>
</Sections>
