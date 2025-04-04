<script lang="ts">
  import { Option } from "effect"
  import { lockedTransferStore } from "../locked-transfer.svelte.ts"
  import { ApprovalRequired } from "../transfer-step.ts"
  import Button from "$lib/components/ui/Button.svelte"
  import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
  import { goto } from "$app/navigation"
  import { truncate } from "$lib/utils/format.ts"
  import { fly } from "svelte/transition"

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

<div class="min-w-full p-6 flex flex-col justify-between h-full">
  <div class="relative overflow-hidden flex-1">
    {#if Option.isSome(transferHashStore.data)}
      <div
              class="absolute inset-0 flex flex-col"
              in:fly={{ x: 20, duration: 300, opacity: 0 }}
              out:fly={{ x: -20, duration: 300, opacity: 0 }}
      >
        <div class="flex-1 flex items-center justify-center">
          <div class="text-center">
            <div class="flex justify-center mb-3">
              <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-union" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-xl font-bold mb-1 text-zinc-400">Transfer Successful!</h3>
            <p class="text-sm text-zinc-500">Your packet has been indexed and confirmed</p>
          </div>
        </div>

        <div class="flex flex-col justify-between gap-3 mt-6">
          <Button onclick={newTransfer} variant="secondary" class="flex-1 py-3 rounded-lg hover:bg-gray-100 transition duration-200 border border-gray-300">
            New transfer
          </Button>
          <Button onclick={handleRedirect} variant="primary" class="flex-1 py-3 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 transition duration-200">
            Check on explorer
          </Button>
        </div>
      </div>
    {:else}
      <div
              class="absolute inset-0 flex flex-col"
              in:fly={{ x: 20, duration: 300, opacity: 0 }}
              out:fly={{ x: -20, duration: 300, opacity: 0 }}
      >
        <div class="flex flex-col items-center justify-center h-full py-8">
          <div class="animate-spin rounded-full h-12 w-12 border-y-2 border-union mb-4"></div>
          <p class="text-lg font-medium text-zinc-400 mb-2">Waiting for indexer...</p>
          <p class="text-sm text-zinc-500 font-mono px-3 py-1 rounded">{truncate(transferHashStore.hash, 8, "middle")}</p>
        </div>
      </div>
    {/if}
  </div>
</div>