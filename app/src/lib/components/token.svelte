<script lang="ts">
import type { Chain } from "$lib/types"
import TokenQualityLevel from "$lib/components/token-quality-level.svelte"
import Truncate from "./truncate.svelte"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"
import { toDisplayName } from "$lib/utilities/chains.ts"
import { formatUnits } from "viem"
import LoadingDots from "./loading-dots.svelte"
import { highlightItem } from "$lib/stores/highlight"
import { cn } from "$lib/utilities/shadcn"
import { derived } from "svelte/store"
import { requestTokenInfo, tokenInfos } from "$lib/stores/tokens"
import { onMount } from "svelte"

export let chains: Array<Chain>
export let chainId: string
export let denom: string
export let amount: string | number | bigint | null = null
export let userAmount: string | null = null
export let expanded = false

let tokenInfo = derived(tokenInfos, $tokenInfos => $tokenInfos[chainId]?.[denom] ?? null)

onMount(() => {
  let chain = chains.find(c => c.chain_id === chainId) ?? null
  if (!chain) {
    console.error("invalid chain in token component")
    return
  }
  requestTokenInfo(chain, denom)
})
</script>

{#if $tokenInfo?.kind === "tokenInfo" && $tokenInfo.info != null}
  {@const token = $tokenInfo.info}
  <!-- svelte-ignore a11y-interactive-supports-focus -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
          class="flex flex-col gap-1"
          on:mouseleave={() => highlightItem.set(null)}
          on:mouseenter={() => {
  highlightItem.set(denom ? { kind: "token", denom} : null)
  }}>
    <div class="flex gap-1 items-center">
      <TokenQualityLevel level={token.graphql != null ? "GRAPHQL" : token.onchain != null ? "ONCHAIN" : "NONE"}/>
      {#if amount !== null}
        {formatUnits(BigInt(amount), token.combined.decimals)}
      {/if}
      {#if userAmount !== null}
        {userAmount}
      {/if}
      <span class={cn("inline-flex gap-1 transition-colors", $highlightItem?.kind === "token" && $highlightItem.denom === denom  ? "bg-union-accent-300 dark:bg-union-accent-950" : "")}><b><Truncate
              value={token.combined.symbol} type="symbol"/></b>
    <div class="text-muted-foreground text-xs flex gap-1 items-center">
      {toDisplayName(chainId, chains)}
      {#each token.combined.wrapping as wrapping}
        <ArrowLeftIcon/>{toDisplayName(
        wrapping.unwrapped_chain.chain_id,
        chains,
      )}
      {/each}
    </div></span>
    </div>
    {#if expanded}
      <div class="text-xs flex flex-col gap gap-4 text-muted-foreground">
        <section>
          <h2 class="text-foreground">Denom</h2>
          <div>
            <Truncate value={denom} type="address"/>
          </div>
        </section>
        {#if token.graphql}
          <section>
            <h2 class="text-foreground">GrapqhQL</h2>
            <div>Name: {token.graphql.primaryRepresentation.name}</div>
            <div>Symbol: {token.graphql.primaryRepresentation.symbol}</div>
            <div>Decimals: {token.graphql.primaryRepresentation.decimals}</div>
            {#if token.graphql.cw20}blah{/if}
            {#if token.graphql.primaryRepresentation.sources}
              <div>Sources:
                {#each token.graphql.primaryRepresentation.sources as source}<a class="underline"
                                                                                href={source.source.source_uri}> {source.source.name}</a>{/each}
              </div>
            {/if}
          </section>
        {/if}
        {#if token.onchain}
          <section>
            <h2 class="text-foreground">Onchain</h2>
            <div>Name: {token.onchain.name}</div>
            <div>Symbol: {token.onchain.symbol}</div>
            <div>Decimals: {token.onchain.decimals}</div>
          </section>
        {/if}
      </div>
    {/if}
  </div>
{:else}
  <div class="flex max-h-auto overflow-hidden text-muted-foreground">
    <div class="relative w-12 h-4">
      <LoadingDots class="absolute -top-4 size-12 h-12 w-12"/>
    </div>
    <Truncate value={denom} type="address"/>
  </div>
{/if}
