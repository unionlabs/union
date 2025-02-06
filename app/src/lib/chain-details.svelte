<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types.ts"
// TODO: rename this component
import TokenQualityLevel from "./components/token-quality-level.svelte"
export let chains: Array<Chain>
export let chainId: string
const chain = chains.find(c => c.chain_id === chainId)
</script>

<span class="inline-flex items-center gap-1">
{#if !chain}
  Invalid Chain: {chainId}
{:else}
  {#if chain.relayer_status.status !== "HEALTHY"}<Tooltip.Root><Tooltip.Trigger><TokenQualityLevel level="NONE"/></Tooltip.Trigger><Tooltip.Content>{chain.relayer_status.message}</Tooltip.Content></Tooltip.Root>{:else}<TokenQualityLevel level="GRAPHQL"/>{/if}
  <div>{chain.display_name}</div>
{/if}
</span>
