<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { Cause, Data, Effect, Exit, Option } from "effect"
import { createWalletClient, type Hash } from "viem"
import { sepolia } from "viem/chains"

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

let error = $state<Option.Option<string>>(Option.none())
let isSubmitting = $state(false)
let txHash = $state<Option.Option<Hash>>(Option.none())

class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: Error
}> {}

class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: Error
}> {}

const submitFlow = Effect.gen(function* () {
  const walletClient = yield* Effect.try({
    try: () =>
      createWalletClient({
        chain: sepolia,
        transport: () => window.ethereum
      }),
    catch: err => new CreateWalletClientError({ cause: err as Error })
  })

  const hash = yield* Effect.tryPromise({
    try: () =>
      walletClient.sendTransaction({
        account: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
        amount: 1,
        to: rawIntents.receiver as `0x${string}`
      }),
    catch: err => new SendTransactionError({ cause: err as Error })
  })
  return hash
})

async function submit() {
  isSubmitting = true
  txHash = Option.none()
  error = Option.none()
  const exit = await Effect.runPromiseExit(submitFlow)
  Exit.match(exit, {
    onFailure: cause => {
      error = Option.some(
        Cause.match(cause, {
          onFail: (err: CreateWalletClientError | SendTransactionError) =>
            err._tag === "CreateWalletClientError"
              ? "could not connect wallet"
              : "could not submit transfer",
          onEmpty: "empty",
          onDie: () => "die",
          onInterrupt: () => "transfer interrupted",
          onSequential: () => "blah",
          onParallel: () => "blah"
        })
      )
    },
    onSuccess: (hash: `0x${string}`) => {
      txHash = Option.some(hash)
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

      {#if error}
        <div class="text-red-500 mt-2">
          {error}
        </div>
      {/if}

      {#if txHash}
        <div class="text-green-500 mt-2">
          Transaction submitted! Hash: {txHash}
        </div>
      {/if}
    </div>
  </section>
</Sections>
