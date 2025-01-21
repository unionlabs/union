<script lang="ts">
import type { Chain, TokenInfo } from "$lib/types"
import TokenQualityLevel from "$lib/components/token-quality-level.svelte"
import Truncate from "./truncate.svelte"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"
import { toDisplayName } from "$lib/utilities/chains.ts"
import { formatUnits } from "viem"

export let chains: Array<Chain>
export let chainId: string
export let denom: string
export let amount: string | number | bigint | null = null
export let expanded: bool = false

let chain = chains.find(c => c.chain_id === chainId) ?? null
let graphqlToken = chain?.tokens.find(t => t.denom === denom) ?? null

let token: TokenInfo = (() => {
  let graphqlToken = chain?.tokens.find(t => t.denom === denom) ?? null

  if (graphqlToken?.representations && graphqlToken.representations.length > 0) {
    let fullRepresentations = graphqlToken.representations.filter(
      repr => repr.decimals != null && repr.name != null && repr.symbol != null
    ) as Array<
      {
        decimals: number
        name: string
        symbol: string
      } & (typeof graphqlToken.representations)[number]
    >

    if (fullRepresentations.length > 0) {
      return {
        quality_level: "GRAPHQL",
        denom,
        primaryRepresentation: fullRepresentations[0],
        representations: fullRepresentations,
        wrapping: graphqlToken.wrapping
      }
    }
  }

  return {
    quality_level: "NONE",
    denom
  }
})()
</script>

<div>
  <div class="flex gap-1 items-center">
    <TokenQualityLevel level={token.quality_level} />
    {#if token.quality_level === "GRAPHQL"}
      {#if amount !== null}
        {formatUnits(BigInt(amount), token.primaryRepresentation.decimals)}
      {/if}
      <div class="font-bold">{token.primaryRepresentation.symbol}</div>
      <div class="text-muted-foreground text-xs flex gap-1 items-center">
        {toDisplayName(chainId, chains)}
        {#each token.wrapping as wrapping}
          <ArrowLeftIcon />{toDisplayName(
            wrapping.unwrapped_chain.chain_id,
            chains,
          )}
        {/each}
      </div>
    {:else}
      {amount}
      <b><Truncate value={token.denom} type="address" /></b>
      <div class="text-muted-foreground text-xs flex gap-1 items-center">
        {toDisplayName(chainId, chains)}
      </div>
    {/if}
  </div>
  {#if expanded}
    <div class="text-xs">
      {#if token.quality_level === "GRAPHQL"}
        <div>Name: {token.primaryRepresentation.name}</div>
        <div>Symbol: {token.primaryRepresentation.symbol}</div>
        <div>Denom: <Truncate value={token.denom} type="address" /></div>
        <div>Amount: {amount}</div>
        <div>Decimals: {token.primaryRepresentation.decimals}</div>
        <div>
          Verifiers: {#each token.primaryRepresentation.sources as source}<a
              class="underline"
              href={source.source.source_uri}>{source.source.name}</a
            >{/each}
        </div>
      {:else}
        <div>Denom: <Truncate value={token.denom} type="address" /></div>
        <div>Amount: {amount}</div>
      {/if}
    </div>
  {/if}
</div>
