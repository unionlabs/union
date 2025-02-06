<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types.ts"
// TODO: rename this component
import TokenQualityLevel from "./components/token-quality-level.svelte"

// you can either pass in Chain or chains = chainId
export let chains: Array<Chain> | null = null
export let chainId: string | null = null
export let chain: Chain | null = null
const ch = chain ? chain : chains?.find(c => c.chain_id === chainId)
</script>

<span class="inline-flex items-center gap-1">
{#if !ch}
  Invalid chain: {#if chainId}{chainId}{/if}
{:else}
  {#if ch.relayer_status.status !== "HEALTHY"}<Tooltip.Root><Tooltip.Trigger><TokenQualityLevel level="NONE"/></Tooltip.Trigger><Tooltip.Content>{ch.relayer_status.message}</Tooltip.Content></Tooltip.Root>{:else}<TokenQualityLevel level="GRAPHQL"/>{/if}
  <div>{ch.display_name}</div>
{/if}
</span>
