<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { truncate } from "$lib/utils/format.ts"
import type { Chain } from "$lib/schema/chain"
import { Option } from "effect"

type Props = {
  token: string
  currentAllowance: bigint
  requiredAmount: bigint
  sourceChain: Chain
  onBack: () => void
  onApprove: () => void
  actionButtonText: string
}

const { 
  token, 
  currentAllowance, 
  requiredAmount, 
  sourceChain,
  onBack, 
  onApprove, 
  actionButtonText 
}: Props = $props()
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex-1">
    <h3 class="text-lg font-semibold mb-4">Approve Token</h3>
    <div class="bg-zinc-800 rounded-lg p-4 mb-4">
      <div class="mb-2">
        <span class="text-zinc-400">Token:</span>
        <span class="font-mono text-sm ml-2">
          <TokenComponent chain={sourceChain} denom={token}/>
        </span>
      </div>
      <div class="mb-2">
        <span class="text-zinc-400">Current Allowance:</span>
        <span class="font-mono text-sm ml-2">{currentAllowance.toString()}</span>
      </div>
      <div>
        <span class="text-zinc-400">Required Amount:</span>
        <span class="font-mono text-sm ml-2">{requiredAmount.toString()}</span>
      </div>
    </div>
    <p class="text-sm text-zinc-400">
      You need to approve the smart contract to spend your tokens.
      This is a one-time approval for this token.
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
      onclick={onApprove}
    >
      {actionButtonText}
    </Button>
  </div>
</div>
