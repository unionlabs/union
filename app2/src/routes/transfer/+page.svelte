<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { Effect, Exit, Option, Either } from "effect"
import { type Hash } from "viem"
import { sepolia } from "viem/chains"
import { submitTransfer, type SubmitTransferError } from "$lib/services/transfer"

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

let isSubmitting = $state(false)
let submissionResult = $state<Option.Option<Either.Either<Hash, SubmitTransferError>>>(
  Option.none()
)

async function submit() {
  isSubmitting = true
  submissionResult = Option.none()
  const exit = await Effect.runPromiseExit(
    submitTransfer({
      chain: sepolia,
      account: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
      value: 1n,
      to: rawIntents.receiver as `0x${string}`
    })
  )
  Exit.match(exit, {
    onFailure: cause => {
      if (cause._tag === "Fail") {
        submissionResult = Option.some(Either.left(cause.error))
      } else {
        console.error("Unexpected causes of program exit", exit)
      }
    },
    onSuccess: (hash: Hash) => {
      submissionResult = Option.some(Either.right(hash))
    }
  })
  isSubmitting = false
}
</script>

<Sections>
  <h1 class="text-2xl font-bold mb-6 text-sky-400">RawIntents Test Page</h1>
  
  <Card>
    <h3 class="text-lg font-semibold mb-2 text-sky-300">Current State:</h3>
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
          disabled={isSubmitting}
        >
          {#if isSubmitting}
            Submitting...
          {:else}
            Submit
          {/if}
        </Button>
        <Button 
          class="mt-4 self-start"
          variant="secondary"
          onclick={resetAll}
          disabled={isSubmitting}
        >
          Reset All
        </Button>
      </div>

      {#if Option.isSome(submissionResult)}
        {#if Either.isRight(submissionResult.value)}
          <div class="text-green-500 mt-2">
            Transaction submitted! Hash: {submissionResult.value.right}
          </div>
        {:else}
          <pre class="text-red-500 mt-2">
            {submissionResult.value.left}
            {JSON.stringify(submissionResult.value.left.cause, null, 2)}
          </pre>
        {/if}
      {/if}
    </div>
  </section>
</Sections>
