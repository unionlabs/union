<script lang="ts">
import StakingListItemComponent from "$lib/components/model/StakingListItemComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import { matchRuntimeResult } from "$lib/utils/snippets.svelte"
import type { Bond, Unbond } from "@unionlabs/sdk/schema/stake"
import { Array as A, Option as O } from "effect"
import type { Exit } from "effect"

interface Props {
  data: O.Option<Exit.Exit<O.Option<A.NonEmptyReadonlyArray<Bond | Unbond>>, any>>
  walletConnected: boolean
}

let { data, walletConnected }: Props = $props()

type TableFilter = "all" | "bond" | "unbond"

let tableFilter = $state<TableFilter>("all")
let currentPage = $state<number>(1)

const itemsPerPage = 10

$effect(() => {
  void tableFilter
  currentPage = 1
})
</script>

{#snippet renderBondsTable(maybeBonds: O.Option<A.NonEmptyReadonlyArray<Bond | Unbond>>)}
  {@const bonds = O.getOrElse(maybeBonds, () => [])}
  {@const filteredBonds = bonds.filter(bond =>
    tableFilter === "all"
    || (tableFilter === "bond" && bond._tag === "Bond")
    || (tableFilter === "unbond" && bond._tag === "Unbond")
  )}
  {@const totalItems = filteredBonds.length}
  {@const totalPages = Math.max(1, Math.ceil(totalItems / itemsPerPage))}
  {@const startIndex = (currentPage - 1) * itemsPerPage}
  {@const endIndex = startIndex + itemsPerPage}
  {@const paginatedBonds = filteredBonds.slice(startIndex, endIndex)}
  {@const hasData = O.isSome(maybeBonds) && filteredBonds.length > 0}

  <div class="relative">
    <div class="p-4 border-b border-zinc-800">
      <div class="flex items-center justify-between gap-1 sm:gap-2">
        <Tabs
          items={[
            { id: "all", label: "All" },
            { id: "bond", label: "Stakes" },
            { id: "unbond", label: "Unstakes" },
          ]}
          activeId={tableFilter}
          onTabChange={(id) => tableFilter = id as TableFilter}
        />

        {#if hasData && totalPages > 1}
          <div class="flex gap-0.5 sm:gap-1">
            <button
              onclick={() => currentPage = Math.max(1, currentPage - 1)}
              disabled={currentPage <= 1}
              class="px-1 sm:px-2 py-1 text-xs sm:text-sm font-medium rounded transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-zinc-500 hover:text-zinc-300 w-5 sm:w-8 bg-zinc-900 hover:bg-zinc-800"
            >
              ←
            </button>
            <div class="px-1.5 sm:px-3 py-1 text-xs sm:text-sm font-medium rounded text-white bg-zinc-800 min-w-[1.5rem] sm:min-w-[3rem] text-center">
              {currentPage}/{totalPages}
            </div>
            <button
              onclick={() => currentPage = Math.min(totalPages, currentPage + 1)}
              disabled={currentPage >= totalPages}
              class="px-1 sm:px-2 py-1 text-xs sm:text-sm font-medium rounded transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed text-zinc-500 hover:text-zinc-300 w-5 sm:w-8 bg-zinc-900 hover:bg-zinc-800"
            >
              →
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="relative overflow-auto">
      {#if hasData}
        {#each paginatedBonds as item}
          <StakingListItemComponent {item} />
        {/each}
      {:else}
        {#each Array(5) as _}
          <div class="p-4 border-b border-zinc-800/50 last:border-b-0">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 bg-zinc-700/50 rounded animate-pulse"></div>
                <div class="flex flex-col gap-1">
                  <div class="w-24 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                  <div class="w-16 h-3 bg-zinc-700/50 rounded animate-pulse"></div>
                </div>
              </div>
              <div class="flex flex-col items-end gap-1">
                <div class="w-20 h-4 bg-zinc-700/50 rounded animate-pulse"></div>
                <div class="w-12 h-3 bg-zinc-700/50 rounded animate-pulse"></div>
              </div>
            </div>
          </div>
        {/each}
        
        <div class="absolute inset-0 flex items-center justify-center backdrop-blur-sm bg-zinc-950/60">
          <div class="text-center">
            {#if !walletConnected}
              <div class="text-zinc-400 text-sm font-medium">
                Connect wallet to view history
              </div>
              <div class="text-zinc-500 text-xs mt-1">
                Your staking transactions will appear here
              </div>
            {:else}
              <div class="text-zinc-400 text-sm font-medium">
                No {
                  tableFilter === "all"
                  ? "transactions"
                  : tableFilter === "bond"
                  ? "stake transactions"
                  : "unstake transactions"
                } yet
              </div>
              <div class="text-zinc-500 text-xs mt-1">
                Your staking history will appear here
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet renderSkeleton()}
  <div class="relative overflow-auto max-h-72 rounded-lg ring-1 ring-zinc-800/80 animate-pulse">
    <table class="w-full text-sm">
      <thead class="sticky top-0 z-10 bg-zinc-950/90">
        <tr class="text-zinc-400 border-b border-zinc-800/80">
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">arrow</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Type</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Chain</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">
            Timestamp
          </th>
          <th class="px-3 py-2 text-right font-semibold tracking-wide text-xs uppercase">Amount</th>
          <th class="px-3 py-2 text-left font-semibold tracking-wide text-xs uppercase">Status</th>
        </tr>
      </thead>
      <tbody>
        {#each Array(10) as _}
          <tr class="even:bg-zinc-900/30 odd:bg-zinc-900/10">
            <td class="px-3 py-2"><div class="h-4 w-24 bg-zinc-700/50 rounded"></div></td>
            <td class="px-3 py-2"><div class="h-4 w-32 bg-zinc-700/50 rounded"></div></td>
            <td class="px-3 py-2 text-right">
              <div class="h-4 w-16 bg-zinc-700/50 rounded ml-auto"></div>
            </td>
            <td class="px-3 py-2"><div class="h-4 w-14 bg-zinc-700/50 rounded"></div></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/snippet}

{#snippet renderError(error: any)}
  <pre class="text-red-500 overflow-auto">{JSON.stringify(error, null, 2)}</pre>
{/snippet}

<Card
  class="p-0"
  divided
>
  {#if walletConnected}
    {@render matchRuntimeResult(data, {
      onSuccess: renderBondsTable,
      onFailure: renderError,
      onNone: renderSkeleton,
    })}
  {:else}
    {@render renderBondsTable(O.none())}
  {/if}
</Card>