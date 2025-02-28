<script lang="ts">
import type { Chain } from "$lib/schema/chain"
import type { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { Option } from "effect"
import Truncate from "$lib/components/ui/Truncate.svelte"

interface Props {
  chain: Chain
  denom: typeof TokenRawDenom.Type
  amount?: typeof TokenRawAmount.Type
}

const { chain, denom, amount = undefined }: Props = $props()

// TODO: Add token symbol/name lookup based on chain and denom
const displayDenom = denom

// TODO: format amount based on info from chain
const displayAmount = $derived(Option.fromNullable(amount).pipe(Option.map(amt => amt.toString())))
</script>

<div class="flex items-center gap-1">
  <Truncate value={displayDenom} maxLength={10} />
  {#if Option.isSome(displayAmount)}
    <span class="font-mono">({displayAmount.value})</span>
  {/if}
</div>
