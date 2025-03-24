<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { Data, Effect, Exit, Option } from "effect"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { ApprovalRequired } from "../transfer-step.ts"
import type { switchChain } from "$lib/services/transfer-ucs03-evm/chain.ts"
import type { writeContract } from "@unionlabs/sdk/evm"
import type { waitForTransferReceipt } from "$lib/services/transfer-ucs03-evm/transactions.ts"
import type { Hash } from "viem"

type Props = {
  stepIndex: number
  onBack: () => void
  onApprove: () => void
  actionButtonText: string
}

/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY
/// BOUNDRY

type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E> : never

export type SwitchChainState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof switchChain>> } // TODO: yield requirements in SwitchChain
}>
export const SwitchChainState = Data.taggedEnum<SwitchChainState>()

export type WriteContractState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof writeContract>> }
}>
export const WriteContractState = Data.taggedEnum<WriteContractState>()

export type TransactionReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Complete: { exit: EffectToExit<ReturnType<typeof waitForTransferReceipt>> }
}>
export const TransactionReceiptState = Data.taggedEnum<TransactionReceiptState>()

export type TransactionSubmissionEvm = Data.TaggedEnum<{
  Filling: {}
  SwitchChain: { state: SwitchChainState }
  WriteContract: { state: WriteContractState }
  TransactionReceipt: { state: TransactionReceiptState }
}>

export const TransactionSubmissionEvm = Data.taggedEnum<TransactionSubmissionEvm>()

export const nextState = (ts: TransactionSubmissionEvm): TransactionSubmissionEvm =>
  TransactionSubmissionEvm.$match(ts, {
    Filling: () => TransactionSubmissionEvm.SwitchChain({ state: SwitchChainState.InProgress() }),
    SwitchChain: ({ state }) =>
      SwitchChainState.$match(state, {
        InProgress: () => {
          // TODO: switch the chain
          return ts
        },
        Complete: ({ exit }) =>
          exit._tag === "Failure"
            ? TransactionSubmissionEvm.SwitchChain({ state: SwitchChainState.InProgress() })
            : TransactionSubmissionEvm.WriteContract({ state: WriteContractState.InProgress() })
      }),
    WriteContract: ({ state }) =>
      WriteContractState.$match(state, {
        InProgress: () => {
          // TODO: Write the contract
          return ts
        },
        Complete: ({ exit }) =>
          exit._tag === "Failure"
            ? TransactionSubmissionEvm.WriteContract({ state: WriteContractState.InProgress() })
            : TransactionSubmissionEvm.TransactionReceipt({
                state: TransactionReceiptState.InProgress({ hash: exit.value })
              })
      }),
    TransactionReceipt: ({ state }) =>
      TransactionReceiptState.$match(state, {
        InProgress: () => {
          // TODO: wait for receipt
          return ts
        },
        Complete: () => ts // There is no next state, return self
      })
  })

/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY
/// END BOUNDRY

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
        onclick={onApprove}
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
