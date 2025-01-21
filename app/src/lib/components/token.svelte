<script lang="ts">
import type { Chain, TokenInfo } from "$lib/types"
import TokenQualityLevel from "$lib/components/token-quality-level.svelte"
import Truncate from "./truncate.svelte"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"
import { toDisplayName } from "$lib/utilities/chains.ts"

export let chains: Array<Chain>
export let chainId: string
export let denom: string

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

<div class="flex gap-1 items-center">
  {#if token.quality_level === "GRAPHQL"}
    <div class="font-bold">{token.primaryRepresentation.symbol}</div>
    <div class="text-muted-foreground text-xs flex gap-1 items-center">
    {#each token.wrapping as wrapping}
       <ArrowLeftIcon/>{toDisplayName(wrapping.wrapped_chain.chain_id, chains)}
    {/each}
    </div>
  {:else}
    <Truncate value={token.denom} type = "address"/>
  {/if}
  <TokenQualityLevel level={token.quality_level}/>
</div>
