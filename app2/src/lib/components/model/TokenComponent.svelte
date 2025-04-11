<script lang="ts">
import { getChain, TokenRawAmount, type Chain, type TokenRawDenom } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import Truncate from "$lib/components/ui/Truncate.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import { chains } from "$lib/stores/chains.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ArrowDownLeft from "$lib/components/icons/ArrowDownLeft.svelte"
import SharpArrowLeft from "$lib/components/icons/SharpArrowLeft.svelte"
import A from "../ui/A.svelte"
import SharpRightArrowIcon from "../icons/SharpRightArrowIcon.svelte"
import Label from "../ui/Label.svelte"
import LongMonoWord from "../ui/LongMonoWord.svelte"
import { fromHex } from "viem"

interface Props {
  chain: Chain
  denom: TokenRawDenom
  amount?: TokenRawAmount
  showWrapping?: boolean
  icon?: Option.Option<string> | undefined
}

const {
  chain,
  denom,
  amount = undefined,
  showWrapping = true,
  icon = Option.none()
}: Props = $props()

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
      if (!amt) return Option.some("0")
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
    <div class="flex items-center gap-2 font-semibold">
      {#if Option.isSome(icon)}
        <img class="size-4" src={icon.value} alt="" />
      {/if}
      {#if amount !== undefined}
        <span>
          {Option.getOrElse(displayAmount, () =>
            amount === undefined ? "" : amount.toString(),
          )}
        </span>
      {/if}
      <Truncate value={displayDenom} maxLength={10} showCopy={false} />
    </div>

    {#if Option.isSome(chains.data) && Option.isSome(token) && showWrapping}
      <div class="text-xs text-zinc-400 flex gap-1">
        {#each token.value.wrapping as wrap, i}
          {#if i === 0}<ArrowDownLeft class="size-3 rotate-90" />{/if}
          {@const wrapChain = getChain(
            chains.data.value,
            wrap.unwrapped_chain.universal_chain_id,
          )}
          {#if Option.isSome(wrapChain)}
            {#if i != 0}<SharpRightArrowIcon class="size-4 rotate-180" />{/if}
            <div><ChainComponent chain={wrapChain.value} /></div>
          {/if}
        {/each}
      </div>
    {/if}
  {/snippet}

  {#snippet content()}
    {#if Option.isSome(token)}
      <div class="text-sm flex flex-col gap-4 text-neutral-400 text-left">
        <section class="flex flex-col">
          {#if token.value.representations.length > 0}
            <h2 class="text-white font-bold text-lg">
              {token.value.representations[0].symbol}
            </h2>
            <span class="text-neutral-500"> </span>
          {/if}
          {#if Option.isSome(chains.data)}
            <div class="text-xs text-zinc-400 flex gap-1">
              {#each token.value.wrapping as wrap, i}
                {#if i === 0}<ArrowDownLeft class="size-3 rotate-90" />{/if}

                {@const wrapChain = getChain(
                  chains.data.value,
                  wrap.unwrapped_chain.universal_chain_id,
                )}
                {#if Option.isSome(wrapChain)}
                  {#if i !== 0}<SharpRightArrowIcon
                      class="size-4 rotate-180"
                    />{/if}
                  <div><ChainComponent chain={wrapChain.value} /></div>
                {/if}
              {/each}
            </div>
          {/if}
        </section>

        <section>
          <Label>Chain</Label>
          <ChainComponent {chain} />
        </section>
        <section>
          <Label>Raw Denom</Label>
          <LongMonoWord>{denom}</LongMonoWord>
        </section>
        {#if chain.rpc_type === "cosmos"}
          <section>
            <Label>Denom</Label>
            <LongMonoWord>{fromHex(denom, "string")}</LongMonoWord>
          </section>
        {/if}

        {#each token.value.representations as rep}
          <section>
            <Label>Name</Label>
            <div>{rep.name}</div>
          </section>
          <section>
            <Label>Symbol</Label>
            <div>{rep.symbol}</div>
          </section>
          <section>
            <Label>Decimals</Label>
            <div>{rep.decimals}</div>
          </section>
          <section>
            <Label>Rank</Label>
            <div>{Option.getOrElse(token.value.rank, () => "Unranked")}</div>
          </section>
          <section>
            {#each rep.sources as source}
              {#if source.source.source_uri}
                <section>
                  <Label>Source</Label>
                  <A class="underline" href={source.source.source_uri}
                    >{source.source.name}</A
                  >
                </section>
              {/if}
            {/each}
          </section>
        {/each}
      </div>
    {/if}
  {/snippet}
</Tooltip>
