<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { Effect, Match, Option } from "effect"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { ApprovalRequired } from "../transfer-step.ts"
import { createViemPublicClient } from "@unionlabs/sdk/evm"
import { erc20Abi, http } from "viem"
import {
  nextStateEvm,
  hasFailedExit,
  isComplete,
  TransactionSubmissionEvm
} from "$lib/components/Transfer/state/evm.ts"
import { getWalletClient } from "$lib/services/evm/clients.ts"
import { TransactionSubmissionCosmos } from "$lib/components/Transfer/state/cosmos.ts"

type Props = {
  stepIndex: number
  onBack: () => void
  onApprove: () => void
  actionButtonText: string
}

const { stepIndex, onBack, onApprove, actionButtonText }: Props = $props()

const lts = lockedTransferStore.get()

// Get the step data from the locked transfer store
const step: Option.Option<ReturnType<typeof ApprovalRequired>> = $derived.by(() => {
  if (Option.isNone(lts)) return Option.none()

  const steps = lts.value.steps
  if (stepIndex < 0 || stepIndex >= steps.length) return Option.none()

  const step = steps[stepIndex]
  return step._tag === "ApprovalRequired" ? Option.some(step) : Option.none()
})

const sourceChain = $derived(lts.pipe(Option.map(ltss => ltss.sourceChain)))

let ets = $state<TransactionSubmissionEvm>(TransactionSubmissionEvm.Filling())
let cts = $state<TransactionSubmissionCosmos>(TransactionSubmissionCosmos.Filling())

const submit = Effect.gen(function* () {
  if (Option.isNone(step) || Option.isNone(lts)) return

  const sourceChainRpcType = lts.value.sourceChain.rpc_type

  yield* Match.value(sourceChainRpcType).pipe(
    Match.when("evm", () =>
      Effect.gen(function* () {
        // Use the component-level state variable
        const viemChain = lts.value.sourceChain.toViemChain()
        if (Option.isNone(viemChain)) return Effect.succeed(null)

        const publicClient = yield* createViemPublicClient({
          chain: viemChain.value,
          transport: http()
        })

        const walletClient = yield* getWalletClient(lts.value.sourceChain)

        do {
          ets = yield* Effect.tryPromise({
            try: () =>
              nextStateEvm(ets, viemChain.value, publicClient, walletClient, {
                chain: viemChain.value,
                account: walletClient.account,
                address: step.value.token,
                abi: erc20Abi,
                functionName: "approve",
                args: [lts.value.channel.source_port_id, step.value.requiredAmount]
              }),
            catch: error => (error instanceof Error ? error : new Error("Unknown error"))
          })

          if (isComplete(ets)) {
            onApprove()
            break
          }
        } while (!hasFailedExit(ets))

        return Effect.succeed(ets)
      })
    ),
    Match.when("cosmos", () =>
      Effect.gen(function* () {
        yield* Effect.log("doing cosmos")
        return Effect.succeed(cts)
      })
    ),
    Match.orElse(() =>
      Effect.gen(function* () {
        yield* Effect.log("unknown chain type")
        return Effect.succeed("unknown chain type")
      })
    )
  )
})
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && Option.isSome(sourceChain)}
    <div class="flex-1">
      <h3 class="text-lg font-semibold mb-4">Approve Token</h3>
      <div class="bg-zinc-800 rounded-lg p-4 mb-4">
        <div class="mb-2">
          <span class="text-zinc-400">Token:</span>
          <span class="font-mono text-sm ml-2">
            <TokenComponent chain={sourceChain.value} denom={step.value.token}/>
          </span>
        </div>
        <div class="mb-2">
          <span class="text-zinc-400">Current Allowance:</span>
          <span class="font-mono text-sm ml-2">{step.value.currentAllowance.toString()}</span>
        </div>
        <div>
          <span class="text-zinc-400">Required Amount:</span>
          <span class="font-mono text-sm ml-2">{step.value.requiredAmount.toString()}</span>
        </div>
      </div>
      <p class="text-sm text-zinc-400">
        You need to approve the smart contract to spend your tokens.
        This is a one-time approval for this token.
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
      <p class="text-zinc-400">Loading approval details...</p>
    </div>
  {/if}
</div>
