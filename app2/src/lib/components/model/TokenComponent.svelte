<script lang="ts">
import type { Chain, UniversalChainId } from "$lib/schema/chain"
import type { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { Effect, Option } from "effect"
import Truncate from "$lib/components/ui/Truncate.svelte"
import { tokensQuery } from "$lib/queries/tokens.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"

interface Props {
  chain: Chain
  denom: TokenRawDenom
  amount?: TokenRawAmount
}

const { chain, denom, amount = undefined }: Props = $props()

// Start the query when the component mounts
$effect(() => {
  tokensStore.fetchTokens(chain.universal_chain_id)
})

// Get token info from store
const token = $derived(
  tokensStore
    .getData(chain.universal_chain_id)
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
</script>

<Tooltip>
  {#snippet trigger()}
    <div class="flex items-center gap-1 font-semibold">
      {#if Option.isSome(displayAmount)}
        <span>{displayAmount.value}</span>
      {/if}
      <Truncate value={displayDenom} maxLength={10} showCopy={false} />
    </div>
  {/snippet}
  
  {#snippet content()}
    {#if Option.isSome(token)}
        <div class="text-sm flex flex-col gap-4 text-neutral-400">
          <section class="flex justify-between items-center">
            {#if token.value.representations.length > 0}
              <h2 class="text-white font-bold text-lg">{token.value.representations[0].symbol}</h2>
            {/if}
            <div class="bg-sky-400 text-black font-bold rounded px-1">
              {Option.isSome(token.value.cw20) ? "?" : "??"}
            </div>
          </section>
          <section>
            {chain.universal_chain_id}
            {#each token.value.wrapping as wrap}
              ← {wrap.unwrapped_chain.universal_chain_id}
            {/each}
          </section>

          <section>
            <h3 class="text-white">Denom</h3>
            {#if Option.isSome(token.value.cw20)}
              <div>{token.value.cw20.value.cw20_token_address}</div>
            {/if}
            <div>{token.value.denom}</div>
          </section>

          {#each token.value.representations as rep}
            <section>
              <div>Name: {rep.name}</div>
              <div>Symbol: {rep.symbol}</div>
              <div>Decimals: {rep.decimals}</div>
              {#each rep.sources as source}
                {#if source.source.source_uri}
                  <div>
                    Source: <a class="underline" href={source.source.source_uri}>{source.source.name}</a>
                  </div>
                {/if}
              {/each}
            </section>
          {/each}

        </div>
        {/if}
  {/snippet}
</Tooltip>
