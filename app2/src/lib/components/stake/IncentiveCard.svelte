<script lang="ts">
// WORK IN PROGRESS
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import { formatIncentive } from "$lib/services/incentive"
import { matchRuntimeResult } from "$lib/utils/snippets.svelte"
import type { Exit, Option } from "effect"

interface Props {
  incentives: Option.Option<Exit.Exit<any, any>>
}

let { incentives }: Props = $props()

let selectedTab: "incentive" | "rewards" = $state("incentive")

$effect(() => {
  console.log("IncentiveCard received incentives:", incentives)
})

function formatPercentage(value: number): string {
  return formatIncentive(value)
}

function formatLargeNumber(value: number): string {
  if (value >= 1_000_000_000) {
    return `${(value / 1_000_000_000).toFixed(1)}B`
  } else if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`
  } else if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`
  }
  return value.toLocaleString()
}
</script>

{#snippet renderIncentiveData(data: any)}
  <div class="flex flex-col h-full">
    <!-- Header with Tabs -->
    <div class="p-2 border-b border-zinc-800">
      <Tabs
        items={[
          { id: "incentive", label: "Incentive" },
        ]}
        activeId={selectedTab}
        onTabChange={(id) => selectedTab = id as "incentive" | "rewards"}
      />
    </div>

    <!-- Content -->
    <div class="p-2 flex flex-col flex-1">
      {#if selectedTab === "incentive"}
        <!-- Incentive Tab Content (APY equivalent) -->
        <div class="flex justify-center mb-2">
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg w-full">
            <div class="text-4xl font-bold text-accent mb-2">
              {formatPercentage(data.rates.yearly)}
            </div>
            <div class="text-sm text-zinc-400">Annual Compounded Rate</div>
          </div>
        </div>

        <!-- 2x2 Grid of All Metrics -->
        <div class="grid grid-cols-2 gap-2 flex-1">
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-mono text-white mb-1">
              {formatLargeNumber(data.totalSupply)}
            </div>
            <div class="text-xs text-zinc-500">Total Supply</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-mono text-white mb-1">
              {formatLargeNumber(data.bondedTokens)}
            </div>
            <div class="text-xs text-zinc-500">Bonded Tokens</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-semibold text-white mb-1">
              {(data.bondedRatio * 100).toFixed(1)}%
            </div>
            <div class="text-xs text-zinc-500">Bonded Ratio</div>
          </div>
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="text-lg font-semibold text-white mb-1">
              {(data.inflation * 100).toFixed(1)}%
            </div>
            <div class="text-xs text-zinc-500">Inflation Rate</div>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet renderLoading()}
  <div class="flex flex-col h-full">
    <!-- Tabs Skeleton -->
    <div class="pt-2 px-2 pb-2 border-b border-zinc-800">
      <div class="flex gap-1">
        <div class="h-8 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
        <div class="h-8 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
      </div>
    </div>

    <!-- Content Skeleton -->
    <div class="px-3 pb-3 flex flex-col flex-1">
      <!-- Main Display Skeleton -->
      <div class="flex justify-center mb-4 mt-3">
        <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
          <div class="h-12 w-32 bg-zinc-700/50 rounded mb-2 animate-pulse"></div>
          <div class="h-4 w-36 bg-zinc-700/50 rounded animate-pulse"></div>
        </div>
      </div>

      <!-- 2x2 Grid Skeleton -->
      <div class="grid grid-cols-2 gap-4 flex-1">
        {#each Array(4) as _}
          <div class="flex flex-col items-center justify-center p-4 bg-zinc-800/30 rounded-lg">
            <div class="h-5 w-16 bg-zinc-700/50 rounded mb-1 animate-pulse"></div>
            <div class="h-3 w-20 bg-zinc-700/50 rounded animate-pulse"></div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/snippet}

{#snippet renderError(error: any)}
  <div>
    <div class="text-center">
      <div class="text-red-400 text-sm mb-2">Failed to load incentive data</div>
      <div class="text-xs text-zinc-500">
        {error?.message || "Unknown error occurred"}
      </div>
    </div>
  </div>
{/snippet}

<Card class="p-0">
  {@render matchRuntimeResult(incentives, {
      onSuccess: renderIncentiveData,
      onFailure: renderError,
      onNone: renderLoading,
    })}
</Card>
