<script lang="ts">
import { Option } from "effect"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { ApprovalRequired } from "../transfer-step.ts"
import Button from "$lib/components/ui/Button.svelte"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { goto } from "$app/navigation"

type Props = {
  stepIndex: number
  newTransfer: () => void
}

const { stepIndex, newTransfer }: Props = $props()

const lts = lockedTransferStore.get()

const step: Option.Option<ReturnType<typeof ApprovalRequired>> = $derived.by(() => {
  if (Option.isNone(lts)) return Option.none()

  const steps = lts.value.steps
  if (stepIndex < 0 || stepIndex >= steps.length) return Option.none()

  const step = steps[stepIndex]
  return step._tag === "ApprovalRequired" ? Option.some(step) : Option.none()
})

$effect(() => {
  if (Option.isSome(transferHashStore.data)) {
    transferHashStore.stopPolling()
  }
})

const handleRedirect = () => {
  if (Option.isSome(transferHashStore.data)) {
    const packet = transferHashStore.data.value
    goto(`/explorer/transfers/${packet}`)
    transferHashStore.reset()
  }
}
</script>


<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(transferHashStore.data)}
    <div class="flex-1 flex items-center justify-center" >
      <h3 class="text-lg font-semibold mb-4">Packet indexed</h3>
    </div>

    <div class="flex flex-col justify-between gap-2">
      <Button onclick={newTransfer} variant="secondary" class="flex-1">
        New transfer
      </Button>
      <Button onclick={handleRedirect} variant="primary" class="flex-1">
        Check on explorer
      </Button>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center h-full">
      <p class="text-zinc-400">Waiting for indexer...</p>
      <p class="text-zinc-400">tx: {transferHashStore.hash}</p>
    </div>
  {/if}
</div>