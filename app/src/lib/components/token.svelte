<script lang="ts">
import type { Chain } from "$lib/types"
import TokenQualityLevel from "$lib/components/token-quality-level.svelte"
import Truncate from "./truncate.svelte"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"
import { toDisplayName } from "$lib/utilities/chains.ts"
import { formatUnits, fromHex, isHex } from "viem"
import LoadingDots from "./loading-dots.svelte"
import { highlightItem } from "$lib/stores/highlight"
import { cn } from "$lib/utilities/shadcn"
import { derived } from "svelte/store"
import { requestTokenInfo, tokenInfos } from "$lib/stores/tokens"
import { onMount } from "svelte"
import * as Tooltip from "$lib/components/ui/tooltip"
import { truncate } from "$lib/utilities/format"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import Address from "./address.svelte"

export let chains: Array<Chain>
export let chainId: string
export let denom: string
export let amount: string | number | bigint | null = null
export let userAmount: string | null = null
export let highlightEnabled = true
export let showWrapping = true
export let stackedView = false

let tokenInfo = derived(
  tokenInfos,
  $tokenInfos => $tokenInfos[chainId]?.[denom.toLowerCase()] ?? null
)

const chain = chains.find(c => c.chain_id === chainId) ?? null
onMount(() => {
  if (!chain) {
    console.error("invalid chain in token component")
    return
  }
  requestTokenInfo(chain, denom.toLowerCase())
})

let cosmosDenom = derived(tokenInfo, $tokenInfo => {
  if (chain?.rpc_type === "cosmos" && isHex(denom)) {
    const hexDecoded = fromHex(denom, "string")
    return { denom: hexDecoded, type: isValidBech32ContractAddress(hexDecoded) ? "CW20" : "BANK" }
  }
  return null
})
</script>

{#if $tokenInfo?.kind === "tokenInfo" && $tokenInfo.info != null}
  {@const token = $tokenInfo.info}
  
<Tooltip.Root>
  <Tooltip.Trigger>

<!-- svelte-ignore a11y-interactive-supports-focus -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
          class="flex flex-col gap-1"
          on:mouseleave={() => highlightItem.set(null)}
          on:mouseenter={() => {
  highlightItem.set(denom ? { kind: "token", denom} : null)
  }}>
    <div class="flex gap-1 items-center">
      {#if amount !== null}
        {formatUnits(BigInt(amount), token.combined.decimals)}
      {/if}
      {#if userAmount !== null}
        {userAmount}
      {/if}
      <span class={cn("inline-flex gap-1", highlightEnabled && $highlightItem?.kind === "token" && $highlightItem.denom === denom  ? "bg-union-accent-300 dark:bg-union-accent-950" : "")}><b>{truncate(token.combined.symbol, 8)}</b>
              {#if showWrapping}
    {#if !stackedView}
    <div class="text-muted-foreground text-xs flex gap-1 items-center">
      {toDisplayName(chainId, chains)}
      {#each token.combined.wrapping as wrapping}
        <ArrowLeftIcon/>{toDisplayName(
        wrapping.unwrapped_chain.chain_id,
        chains,
      )}
      {/each}
    </div>{/if}{/if}
    </span>
    <TokenQualityLevel level={token.graphql != null ? "GRAPHQL" : token.onchain != null ? "ONCHAIN" : "NONE"}/>
    </div>
    {#if stackedView}
    <div class="text-muted-foreground text-xs flex gap-1 items-center -mt-1">
      {toDisplayName(chainId, chains)}
      {#each token.combined.wrapping as wrapping}
        <ArrowLeftIcon/>{toDisplayName(
        wrapping.unwrapped_chain.chain_id,
        chains,
      )}
      {/each}
      </div>
    {/if}
  </div>
  </Tooltip.Trigger>
  <Tooltip.Content>
    <div class="text-xs flex flex-col gap gap-4 text-neutral-400">
      <section>
        <div class="flex justify-between items-center">
          <h2 class="text-white font-bold text-sm">{token.combined.symbol}</h2>
          <div class="bg-union-accent-500 text-black font-bold rounded px-1">
            {#if $cosmosDenom}
              {$cosmosDenom.type}
            {:else}
              ERC20
            {/if}
          </div>
        </div>
        <div class="flex items-center justify-right gap-1">
          {toDisplayName(chainId, chains)}
          {#each token.combined.wrapping as wrapping}
            <ArrowLeftIcon/>{toDisplayName(wrapping.unwrapped_chain.chain_id, chains,)}
          {/each}
        </div>
      </section>
      <section>
        <h3 class="text-white">Denom</h3>
        {#if $cosmosDenom}
          {#if $cosmosDenom.type === "CW20"}
            <Address truncate={false} highlightEnabled={false} address={$cosmosDenom.denom} {chains} {chainId}/>
          {:else}
            <div>{$cosmosDenom.denom}</div>
          {/if}
          <div>{denom}</div>
        {:else}
          <Address truncate={false} highlightEnabled={false} address={denom} {chains} {chainId}/>
        {/if}
      </section>
      {#if token.graphql}
        <section>
          <h3 class="text-white">GraphQL</h3>
          <div>Name: {token.graphql.primaryRepresentation.name}</div>
          <div>Symbol: {token.graphql.primaryRepresentation.symbol}</div>
          <div>Decimals: {token.graphql.primaryRepresentation.decimals}</div>
          {#if token.graphql.primaryRepresentation.sources}
            <div>Sources:
              <ul class="list-disc">
                {#each token.graphql.primaryRepresentation.sources as source}
                  <li class="ml-4 list-item"><a class="underline" href={source.source.source_uri}> {source.source.name}</a></li>
                {/each}
              </ul>
            </div>
          {/if}
        </section>
      {/if}
      {#if token.onchain}
        <section>
          <h3 class="text-white">Onchain</h3>
          <div>Name: {token.onchain.name}</div>
          <div>Symbol: {token.onchain.symbol}</div>
          <div>Decimals: {token.onchain.decimals}</div>
        </section>
      {/if}
    </div>    
  </Tooltip.Content>
</Tooltip.Root>

{:else}
  <div class="flex max-h-auto overflow-hidden text-muted-foreground">
    <div class="relative w-12 h-4">
      <LoadingDots class="absolute -top-4 size-12 h-12 w-12"/>
    </div>
    <Truncate value={denom} type="address"/>
  </div>
{/if}

