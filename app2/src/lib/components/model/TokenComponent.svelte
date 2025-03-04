<script lang="ts">
import type { Chain } from "$lib/schema/chain"
import type { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { Effect, Option } from "effect"
import Truncate from "$lib/components/ui/Truncate.svelte"
import { tokensQuery } from "$lib/queries/tokens.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"

interface Props {
  chain: Chain
  denom: typeof TokenRawDenom.Type
  amount?: typeof TokenRawAmount.Type
}

const { chain, denom, amount = undefined }: Props = $props()

// Start the query when the component mounts
$effect(() => {
  tokensStore.fetchTokens(chain.chain_id)
})

// Get token info from store
const token = $derived(
  tokensStore
    .getData(chain.chain_id)
    .pipe(Option.flatMap(tokens => Option.fromNullable(tokens.find(t => t.denom === denom))))
)

// Get display info from token representations
const displayInfo = $derived(
  Option.map(token, t => {
    const rep = t.representations[0] // Use first representation
    return {
      symbol: rep.symbol,
      decimals: rep.decimals
    }
  })
)

// Format amount using token decimals if available
const displayAmount = $derived(
  Option.match(Option.all([Option.fromNullable(amount), displayInfo]), {
    onNone: () => Option.none(),
    onSome: ([amt, info]) => {
      const decimal = BigInt(10) ** BigInt(info.decimals)
      const whole = amt / decimal
      const fraction = amt % decimal
      return Option.some(`${whole}${fraction > 0 ? `.${fraction}` : ""}`)
    }
  })
)

// Use symbol if available, otherwise truncate denom
const displayDenom = $derived(
  Option.match(displayInfo, {
    onNone: () => denom,
    onSome: info => info.symbol
  })
)
</script>

{token}
<button onclick={() => console.log(tokensStore)}>latest</button>

<div class="flex items-center gap-1">
  {#if Option.isSome(displayAmount)}
    <span>{displayAmount.value}</span>
  {/if}
  <Truncate value={displayDenom} maxLength={10} />
</div>
