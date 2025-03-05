<script lang="ts">
import type { Chain } from "$lib/schema/chain"
import type { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { Effect, Option } from "effect"
import Truncate from "$lib/components/ui/Truncate.svelte"
import { tokensQuery } from "$lib/queries/tokens.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"

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
  Option.flatMap(token, t => {
    if (t.representations.length === 0) {
      return Option.none()
    }
    const rep = t.representations[0] // Use first representation
    return Option.some({
      symbol: rep.symbol,
      decimals: rep.decimals
    })
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

      // Convert fraction to string and remove trailing zeros
      const fractionStr =
        fraction === 0n
          ? ""
          : `.${fraction.toString().padStart(info.decimals, "0").replace(/0+$/, "")}`

      return Option.some(`${whole}${fractionStr}`)
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
function formatTokenDetails(token: NonNullable<typeof token.value>) {
  const details = []

  // Add basic info
  details.push(`Denom: ${token.denom}`)

  // Add CW20 info if present
  if (Option.isSome(token.cw20)) {
    details.push(`CW20 Address: ${token.cw20.value.cw20_token_address}`)
  }

  // Add representation info
  token.representations.forEach(rep => {
    details.push(`${rep.name} (${rep.symbol})`)
  })

  return details.join("\n")
}
</script>


<Tooltip>
  {#snippet trigger()}
    <div class="flex items-center gap-1 font-semibold">
      {#if Option.isSome(displayAmount)}
        <span>{displayAmount.value}</span>
      {/if}
      <Truncate value={displayDenom} maxLength={10} />
    </div>
  {/snippet}
  
  {#snippet content()}
    {Option.match(token, {
      onNone: () => "Loading token details...",
      onSome: formatTokenDetails
    })}
  {/snippet}
</Tooltip>
