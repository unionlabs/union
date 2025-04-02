<script lang="ts">
import { Option } from "effect"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { ApprovalRequired } from "../transfer-step.ts"
import Button from "$lib/components/ui/Button.svelte"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { goto } from "$app/navigation"

type Props = {
  stepIndex: number
}

const { stepIndex }: Props = $props()

const lts = lockedTransferStore.get()

// Get the step data from the locked transfer store
const step: Option.Option<ReturnType<typeof ApprovalRequired>> = $derived.by(() => {
  if (Option.isNone(lts)) return Option.none()

  const steps = lts.value.steps
  if (stepIndex < 0 || stepIndex >= steps.length) return Option.none()

  const step = steps[stepIndex]
  return step._tag === "ApprovalRequired" ? Option.some(step) : Option.none()
})

$effect(() => {
  if (Option.isSome(transferHashStore.data)) {
    const packet = transferHashStore.data.value
    transferHashStore.reset()
    goto(`/explorer/transfers/${packet}`)
  }
})
</script>


<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && Option.isSome(transferHashStore.data)}
    <div class="flex-1">
      <h3 class="text-lg font-semibold mb-4">Packet indexed</h3>
      <p class="text-sm text-zinc-400">Redirecting..</p>
    </div>

    <!-- Here we can do "check on explorer" and "make another transfer"--->
    <!--    <div class="flex justify-between mt-4">-->
    <!--      <Button-->
    <!--              variant="secondary"-->
    <!--      >-->
    <!--        Action 1-->
    <!--      </Button>-->
    <!--      <Button-->
    <!--              variant="primary"-->
    <!--      >-->
    <!--        Action 2-->
    <!--      </Button>-->
    <!--    </div>-->
  {:else if transferHashStore.hash}
    <div class="flex flex-col items-center justify-center h-full">
      <p class="text-zinc-400">Waiting for indexer...</p>
      <p class="text-zinc-400">tx: {transferHashStore.hash}</p>
    </div>
  {/if}
</div>