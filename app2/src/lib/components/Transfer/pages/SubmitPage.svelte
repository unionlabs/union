<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { transfer } from "../transfer.svelte.ts"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { Option } from "effect"
import { SubmitInstruction } from "../transfer-step.ts"

type Props = {
  stepIndex: number
  onBack: () => void
  onSubmit: () => void
  actionButtonText: string
}

const { 
  stepIndex,
  onBack, 
  onSubmit, 
  actionButtonText 
}: Props = $props()

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

// Get the amount from the transfer args
const amount = $derived(() => transfer.args.amount)
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
          <div>Amount: {amount || "0"}</div>
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
        onclick={onSubmit}
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
