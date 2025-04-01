<script lang="ts">
  import {Option} from "effect"
  import { lockedTransferStore } from "../locked-transfer.svelte.ts"
  import { ApprovalRequired } from "../transfer-step.ts"
  import Button from "$lib/components/ui/Button.svelte";
  import {onMount} from "svelte";

  type Props = {
    hash: string
    stepIndex: number
    onBack: () => void
    actionButtonText: string
  }

  const { stepIndex, hash,  onBack, onIndexed, actionButtonText }: Props = $props()

  const lts = lockedTransferStore.get()

  // Get the step data from the locked transfer store
  const step: Option.Option<ReturnType<typeof ApprovalRequired>> = $derived.by(() => {
    if (Option.isNone(lts)) return Option.none()

    const steps = lts.value.steps
    if (stepIndex < 0 || stepIndex >= steps.length) return Option.none()

    const step = steps[stepIndex]
    return step._tag === "ApprovalRequired" ? Option.some(step) : Option.none()
  })

  onMount(() => {

  })
</script>


<div class="min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(step) && hash}
    <div class="flex-1">
      <h3 class="text-lg font-semibold mb-4">Packet indexed</h3>
      <p class="text-sm text-zinc-400">Redirecting..</p>
    </div>

    <div class="flex justify-between mt-4">
      <Button
              variant="secondary"
      >
        Action 1
      </Button>
      <Button
              variant="primary"
      >
        Action 2
      </Button>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center h-full">
      <p class="text-zinc-400">Waiting for indexer...</p>
      <p class="text-zinc-400">tx: {hash}</p>
    </div>
  {/if}
</div>