<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import type { Chain } from "$lib/types.ts"
// TODO: rename this component
import TokenQualityLevel from "./components/token-quality-level.svelte"
import { highlightItem } from "$lib/stores/highlight"
import { cn } from "$lib/utilities/shadcn"

// you can either pass in Chain or chains = chainId
export let chains: Array<Chain> | null = null
export let chainId: string | null = null
export let chain: Chain | null = null
export let highlightEnabled = true
$: ch = chain ? chain : chains?.find(c => c.chain_id === chainId)
$: chain_id = chain?.chain_id ? chain.chain_id : chainId
</script>

<!-- svelte-ignore a11y-interactive-supports-focus -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
<span class="inline-flex items-center"
  on:mouseleave={() => highlightItem.set(null)}
  on:mouseenter={() => {
  highlightItem.set(chain_id ? { kind: "chain", chainId: chain_id} : null)
  }}
>
{#if !ch}
  Invalid chain: {#if chain_id}{chain_id}{/if}
{:else}
    <div class={cn(highlightEnabled && $highlightItem?.kind === "chain" && $highlightItem.chainId === chain_id ? "bg-union-accent-300 dark:bg-union-accent-950" : "", "text-nowrap")}>{ch.display_name}</div>
{#if ch.relayer_status.status !== "HEALTHY"}<Tooltip.Root><Tooltip.Trigger><TokenQualityLevel level={ch.relayer_status.status === "WARNING" ? "ONCHAIN" : "NONE"}/></Tooltip.Trigger><Tooltip.Content>{ch.relayer_status.message}</Tooltip.Content></Tooltip.Root>{/if}
{/if}
</span>
