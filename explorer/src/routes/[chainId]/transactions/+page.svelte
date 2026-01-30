<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache } from "$lib/snippet-cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import { formatTimeAgo, truncateHash, truncateAddress } from "$lib/utils/format"
import { getMsgType, getMsgTypeVariant } from "$lib/utils/messages"
import { indexer } from "$lib/services/indexer-client"
import { urls } from "$lib/utils/urls"
import type { TxResponse, PaginationResponse } from "$lib/types/cosmos"
import CheckIcon from "@lucide/svelte/icons/check"
import XIcon from "@lucide/svelte/icons/x"
import ArrowRightIcon from "@lucide/svelte/icons/arrow-right"
import LoaderIcon from "@lucide/svelte/icons/loader"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Local state for additional loaded transactions
let additionalTxs = $state<TxResponse[]>([])
let isLoadingMore = $state(false)
let hasMore = $state(true)

onMount(() => {
  const interval = setInterval(() => {
    invalidate("txs:data")
  }, 10_000)
  return () => clearInterval(interval)
})

// Load more transactions using indexer
async function loadMore(currentTxs: TxResponse[]) {
  if (isLoadingMore || !hasMore) return

  isLoadingMore = true
  try {
    const allTxs = [...currentTxs, ...additionalTxs]
    const oldestHeight = Math.min(...allTxs.map(tx => parseInt(tx.height)))
    const moreTxs = await indexer.txs(data.chain.universal_chain_id, 50, oldestHeight)
    if (moreTxs.length === 0) {
      hasMore = false
    } else {
      // Convert IndexedTx to TxResponse format
      additionalTxs = [...additionalTxs, ...moreTxs.map(tx => ({
        txhash: tx.hash,
        height: String(tx.height),
        code: tx.code,
        codespace: tx.codespace,
        gas_used: tx.gas_used,
        gas_wanted: tx.gas_wanted,
        timestamp: tx.timestamp,
        raw_log: tx.raw_log,
        tx: {
          body: {
            messages: tx.messages,
            memo: tx.memo,
          },
          auth_info: {
            fee: tx.fee,
          },
        },
      } as TxResponse))]
    }
  } catch (e) {
    console.error("Failed to load more transactions:", e)
  } finally {
    isLoadingMore = false
  }
}


// Extract relevant info from messages
const getTxSummary = (tx: TxResponse) => {
  const msg = tx.tx.body.messages[0] as Record<string, unknown>
  const type = getMsgType(msg as { "@type": string })

  // Try to extract sender/from
  const from = (msg.sender || msg.from_address || msg.delegator_address || msg.voter || msg.proposer) as string | undefined

  // Try to extract receiver/to
  const to = (msg.receiver || msg.to_address || msg.validator_address) as string | undefined

  return { type, from, to }
}
</script>


{#snippet txRow(tx: TxResponse)}
  {@const summary = getTxSummary(tx)}
  {@const msgCount = tx.tx.body.messages.length}
  <a
    href={urls.transaction(tx.txhash)}
    class="grid grid-cols-12 gap-4 px-4 py-3 border-b border-border hover:bg-muted/30 transition-colors items-center"
  >
    <!-- Status -->
    <div class="col-span-1">
      <div class="w-5 h-5 flex items-center justify-center {tx.code === 0 ? 'bg-success/20' : 'bg-destructive/20'}">
        {#if tx.code === 0}
          <CheckIcon class="h-3 w-3 text-success" />
        {:else}
          <XIcon class="h-3 w-3 text-destructive" />
        {/if}
      </div>
    </div>
    <!-- Hash -->
    <div class="col-span-3">
      <span class="font-mono text-xs">{truncateHash(tx.txhash, 12)}</span>
    </div>
    <!-- Type -->
    <div class="col-span-2">
      <div class="flex items-center gap-1">
        <Badge variant={getMsgTypeVariant(summary.type)} class="text-[10px]">{summary.type}</Badge>
        {#if msgCount > 1}
          <span class="text-[10px] text-muted-foreground">+{msgCount - 1}</span>
        {/if}
      </div>
    </div>
    <!-- From/To -->
    <div class="col-span-3">
      <div class="flex items-center gap-1 text-xs text-muted-foreground">
        {#if summary.from}
          <span class="font-mono">{truncateAddress(summary.from, 6)}</span>
        {/if}
        {#if summary.from && summary.to}
          <ArrowRightIcon class="h-3 w-3 shrink-0" />
        {/if}
        {#if summary.to}
          <span class="font-mono">{truncateAddress(summary.to, 6)}</span>
        {/if}
        {#if !summary.from && !summary.to}
          <span>—</span>
        {/if}
      </div>
    </div>
    <!-- Block -->
    <div class="col-span-1 text-center">
      <span class="text-xs font-mono text-muted-foreground">{tx.height}</span>
    </div>
    <!-- Time -->
    <div class="col-span-2 text-right">
      <span class="text-xs text-muted-foreground">{formatTimeAgo(tx.timestamp)}</span>
    </div>
  </a>
{/snippet}

{#snippet txsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Transactions</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(10) as _}
        <Skeleton class="h-10 w-full" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet txsSuccess(result: { tx_responses: TxResponse[]; pagination: PaginationResponse })}
  {@const initialTxs = result.tx_responses ?? []}
  {@const combinedTxs = [...initialTxs, ...additionalTxs]}
  {@const allTxs = combinedTxs.filter((tx, i, arr) => arr.findIndex(t => t.txhash === tx.txhash) === i)}

  <div class="relative border border-border">
    <CornerMarks />

    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Transactions</span>
      <span class="text-[10px] font-mono text-muted-foreground">{allTxs.length} shown</span>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-12 gap-4 px-4 py-2 border-b border-border bg-muted/10 text-xs font-mono uppercase tracking-wider text-muted-foreground">
      <div class="col-span-1">Status</div>
      <div class="col-span-3">Hash</div>
      <div class="col-span-2">Type</div>
      <div class="col-span-3">From / To</div>
      <div class="col-span-1 text-center">Block</div>
      <div class="col-span-2 text-right">Time</div>
    </div>

    <!-- Transaction List -->
    {#each allTxs as tx (tx.txhash)}
      {@render txRow(tx)}
    {/each}

    {#if allTxs.length === 0}
      <div class="px-4 py-12 text-center">
        <p class="text-sm text-muted-foreground">No transactions found</p>
        <p class="text-xs text-muted-foreground mt-1">This chain may have low activity</p>
      </div>
    {/if}

    <!-- Load More Button -->
    {#if allTxs.length > 0 && hasMore}
      <button
        onclick={() => loadMore(initialTxs)}
        disabled={isLoadingMore}
        class="w-full px-4 py-3 text-sm font-mono text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
      >
        {#if isLoadingMore}
          <LoaderIcon class="h-4 w-4 animate-spin" />
          Loading...
        {:else}
          Load More
        {/if}
      </button>
    {/if}
  </div>
{/snippet}

{#snippet txsError(err: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6">
      <p class="text-sm font-medium text-destructive mb-2">Failed to load transactions</p>
      <p class="text-xs text-muted-foreground font-mono">{String(err)}</p>
    </div>
  </div>
{/snippet}

{@render matchPromiseWithCache(data.transactions, {
  cacheKey: `${cachePrefix}txs:list`,
  onLoading: txsLoading,
  onSuccess: txsSuccess,
  onError: txsError,
})}
