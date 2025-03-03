<script lang="ts">
import { RawIntentsStoreSvelte } from "./raw-intents-store.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { Effect, Exit, Data } from "effect"
import { type Hash, type TransactionReceipt } from "viem"
import { sepolia } from "viem/chains"
import {
  submitTransfer,
  switchChain,
  waitForReceipt,
  type SubmitTransferError
} from "$lib/services/transfer"

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

type SwitchChainState = Data.TaggedEnum<{
  InProgress: {}
  Success: {}
  Failure: {
    readonly cause: unknown
  }
}>

type ApprovalSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Success: { readonly hash: Hash }
  Failure: {
    readonly cause: unknown
  }
}>

type ApprovalReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Success: { readonly receipt: TransactionReceipt }
  Failure: {
    readonly cause: unknown
  }
}>

type TransferSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Success: { readonly hash: Hash }
  Failure: {
    readonly cause: unknown
  }
}>

type TransferReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Success: { readonly receipt: TransactionReceipt }
  Failure: {
    readonly cause: unknown
  }
}>

type TransferSubmission2 = Data.TaggedEnum<{
  Pending: {}
  SwitchChain: { state: SwitchChainState }
  ApprovalSubmit: { state: ApprovalSubmitState }
  ApprovalReceipt: { state: ApprovalReceiptState }
  TransferSubmit: { state: TransferSubmitState }
  TransferReceipt: { state: TransferReceiptState }
}>

type TransferSubmission = Data.TaggedEnum<{
  Pending: {}
  SwitchChain: { state: SwitchChainState }
  TransferSubmit: { state: TransferSubmitState }
  TransferReceipt: { state: TransferReceiptState }
}>

const transferSubmission = Data.taggedEnum<TransferSubmission>()
const switchChainn = Data.taggedEnum<SwitchChainState>()

let transferState = $state<TransferSubmission>(transferSubmission.Pending())

async function submit() {
  transferState = transferSubmission.SwitchChain({ state: switchChainn.InProgress() })

  const switchChainExit = await Effect.runPromiseExit(switchChain(sepolia.id))

  const submissionExit = await Effect.runPromiseExit(
    submitTransfer({
      chain: sepolia,
      account: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
      value: 1n,
      to: rawIntents.receiver as `0x${string}`
    })
  )

  if (Exit.isFailure(submissionExit)) {
    // update state machine
    return
  }

  const receiptExit = await Effect.runPromiseExit(waitForReceipt(submissionExit.value))

  if (Exit.isFailure(receiptExit)) {
    // update state machine
    return
  }

  // Exit.match(exit, {
  //   onFailure: cause => {
  //     if (cause._tag === "Fail") {
  //       if (cause.error._tag === "SendTransactionError") {
  //         transferSubmission = Failure({
  //           reason: "Failed to submit your transfer",
  //           message: "This means that the RPCs might be bad",
  //           error: cause.error
  //         })
  //       } else if (cause.error._tag === "CreateWalletClientError") {
  //         transferSubmission = Failure({
  //           reason: "Could not connect to your wallet",
  //           message:
  //             "Make sure you have your wallet connected and check if your wallet has any errors in its UI",
  //           error: cause.error
  //         })
  //       }
  //     } else {
  //       transferSubmission = Interrupted()
  //     }
  //   },
  //   onSuccess: (receipt: TransactionReceipt) => {
  //     transferSubmission = Success({ hash: receipt.transactionHash })
  //   }
  // })
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
          disabled={transferSubmission._tag === "InProgress"}
        >
          {#if transferSubmission._tag === "InProgress"}
            Submitting...
          {:else}
            Submit
          {/if}
        </Button>
        <Button 
          class="mt-4 self-start"
          variant="secondary"
          onclick={resetAll}
          disabled={transferSubmission._tag === "InProgress"}
        >
          Reset All
        </Button>
      </div>

      {#if transferSubmission._tag === "Success"}
        <div class="text-green-500 mt-2">
          Transaction submitted! Hash: {transferSubmission.hash}
        </div>
      {:else if transferSubmission._tag === "Failure"}
        <div class="text-red-500">
          <h2 class="text-red-500 mt-2">{transferSubmission.reason}</h2>
          <pre class="text-red-500 mt-2">
            {JSON.stringify(transferSubmission.error)}
          </pre>
        </div>
      {:else if transferSubmission._tag === "Interrupted"}
        <div class="text-red-500">
          <h2 class="text-red-500 mt-2">This transfer was interrupted</h2>
        </div>
      {/if}
    </div>
  </section>
</Sections>
