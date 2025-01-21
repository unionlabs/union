<script lang="ts">
import type { Chain, TokenInfo } from "$lib/types"
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
        representations: fullRepresentations
      }
    }
  }

  return {
    quality_level: "NONE",
    denom
  }
})()
</script>


{#if token.quality_level === "GRAPHQL"}
  [G] {token.primaryRepresentation.symbol}
{:else}
  [N] {denom}
{/if}
