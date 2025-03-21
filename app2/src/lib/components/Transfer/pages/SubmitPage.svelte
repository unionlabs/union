<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import type { Chain } from "$lib/schema/chain"

type Props = {
  sourceChain: Chain
  destinationChain: Chain
  amount: string | undefined
  onBack: () => void
  onSubmit: () => void
  actionButtonText: string
}

const { 
  sourceChain, 
  destinationChain, 
  amount, 
  onBack, 
  onSubmit, 
  actionButtonText 
}: Props = $props()
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex-1">
    <h3 class="text-lg font-semibold mb-4">Submit Transfer</h3>
    <div class="bg-zinc-800 rounded-lg p-4 mb-4">
      <p class="mb-2">Ready to submit your transfer instruction to the blockchain.</p>
      <div class="text-sm text-zinc-400">
        <div class="mb-1">From: {sourceChain?.display_name || "Unknown"}</div>
        <div class="mb-1">To: {destinationChain?.display_name || "Unknown"}</div>
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
      variant="outline"
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
</div>
