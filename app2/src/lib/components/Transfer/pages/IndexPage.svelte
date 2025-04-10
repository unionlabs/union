<script lang="ts">
import { Option, Struct } from "effect"
import Button from "$lib/components/ui/Button.svelte"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { goto } from "$app/navigation"
import { fly } from "svelte/transition"
import TransactionHashComponent from "$lib/components/model/TransactionHashComponent.svelte"
import { lockedTransferStore } from "../locked-transfer.svelte"

type Props = {
  newTransfer: () => void
}

const { newTransfer }: Props = $props()

$effect(() => {
  if (Option.isSome(transferHashStore.data)) {
    transferHashStore.stopPolling()
  }
})

const lts = lockedTransferStore.get()
const sourceChain = $derived(lts.pipe(Option.map(Struct.get("sourceChain"))))

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
              <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-babylon-orange" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-xl font-bold mb-1 text-zinc-400">Transfer Successful!</h3>
            <p class="text-sm text-zinc-500">Your packet has been indexed and confirmed</p>
          </div>
        </div>

        <div class="flex flex-col justify-between gap-3 mt-6">
          <Button onclick={newTransfer} variant="secondary">
            New transfer
          </Button>
          <Button onclick={handleRedirect} variant="primary">
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
          <div class="animate-spin rounded-full h-12 w-12 border-y-2 border-babylon-orange mb-4"></div>
          <p class="text-lg font-medium text-zinc-400 mb-2">Waiting for indexer...</p>
          {#if Option.isSome(sourceChain)}
            <TransactionHashComponent hash={transferHashStore.hash} chain={sourceChain.value}/>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
