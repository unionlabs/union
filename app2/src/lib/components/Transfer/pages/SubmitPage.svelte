<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { Effect, Option } from "effect"
import { SubmitInstruction } from "../transfer-step.ts"
import { hasFailedExit, isComplete } from "$lib/components/Transfer/state/evm.ts"
import { generateSalt } from "@unionlabs/sdk/utils"
import { encodeAbi } from "@unionlabs/sdk/ucs03"
import { nextStateEvm, TransactionSubmissionEvm } from "$lib/components/Transfer/state/evm.ts"
import { getConnectorClient, type GetConnectorClientErrorType, http } from "@wagmi/core"
import { createViemPublicClient, createViemWalletClient } from "@unionlabs/sdk/evm"
import { ConnectorClientError } from "$lib/services/transfer"
import { wagmiConfig } from "$lib/wallet/evm/wagmi-config.ts"
import { custom } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"

const { stepIndex, onBack, onSubmit, actionButtonText }: Props = $props()

const lts = lockedTransferStore.get()

// Get the step data from the locked transfer store
const step: Option.Option<ReturnType<typeof SubmitInstruction>> = $derived.by(() => {
  if (Option.isNone(lts)) return Option.none()

  const steps = lts.value.steps
  if (stepIndex < 0 || stepIndex >= steps.length) return Option.none()

  const step = steps[stepIndex]
  return step._tag === "SubmitInstruction" ? Option.some(step) : Option.none()
})

const sourceChain = $derived(lts.pipe(Option.map(ltss => ltss.sourceChain)))
const destinationChain = $derived(lts.pipe(Option.map(ltss => ltss.destinationChain)))

type Props = {
  stepIndex: number
  onBack: () => void
  onSubmit: () => void
  actionButtonText: string
}

let ts = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())

export const submit = Effect.gen(function* () {
  if (Option.isNone(step) || Option.isNone(lts)) return

  const viemChain = lts.value.sourceChain.toViemChain()
  if (Option.isNone(viemChain)) return

  const publicClient = yield* createViemPublicClient({
    chain: viemChain.value,
    transport: http()
  })

  const connectorClient = yield* Effect.tryPromise({
    try: () => getConnectorClient(wagmiConfig),
    catch: err => new ConnectorClientError({ cause: err as GetConnectorClientErrorType })
  })

  const walletClient = yield* createViemWalletClient({
    account: connectorClient.account,
    chain: viemChain.value,
    transport: custom(connectorClient)
  })

  do {
    ts = yield* Effect.tryPromise(() =>
      nextStateEvm(ts, viemChain.value, publicClient, walletClient, {
        chain: viemChain.value,
        account: connectorClient.account,
        address: lts.value.channel.source_port_id,
        abi: ucs03ZkgmAbi,
        functionName: "send",
        args: [
          lts.value.channel.source_channel_id,
          0n,
          1000000000000n,
          generateSalt(),
          {
            opcode: step.value.instruction.opcode,
            version: step.value.instruction.version,
            operand: encodeAbi(step.value.instruction)
          }
        ]
      })
    )

    if (isComplete(ts)) {
      onSubmit()
      break
    }
  } while (!hasFailedExit(ts))

  return ts
})
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && Option.isSome(sourceChain) && Option.isSome(destinationChain)}
    <div class="flex-1">
      <h3 class="text-lg font-semibold mb-4">Submit Transfer</h3>
      <div class="bg-zinc-800 rounded-lg p-4 mb-4">
        <p class="mb-2">Ready to submit your transfer instruction to the blockchain.</p>
        <div class="text-sm text-zinc-400">
          <div class="mb-1">From: {sourceChain.value.display_name || "Unknown"}</div>
          <div class="mb-1">To: {destinationChain.value.display_name || "Unknown"}</div>
          <div>Amount: { "0"}</div>
        </div>
      </div>
      <p class="text-sm text-zinc-400">
        This will initiate the transfer on the blockchain.
        You'll need to confirm the transaction in your wallet.
      </p>
    </div>

    <div class="flex justify-between mt-4">
      <Button
              variant="secondary"
              onclick={onBack}
      >
        Back
      </Button>
      <Button
              variant="primary"
              onclick={() => Effect.runPromise(submit)}
      >
        {actionButtonText}
      </Button>
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-zinc-400">Loading submission details...</p>
    </div>
  {/if}
</div>
