<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { formatTimeAgo, truncateHash, truncateAddress } from "$lib/utils/format"
import { indexer } from "$lib/services/indexer-client"
import { buildValidatorMap, prefetchValidatorAvatars, getCachedAvatar, getAvatarCacheVersion, formatAddress } from "$lib/utils/validators.svelte"
import { addressFormat } from "$lib/stores/address-format.svelte"
import { urls } from "$lib/utils/urls"
import type { BlockSummary } from "$lib/queries/blocks"
import type { Validator } from "$lib/types/cosmos"
import LoaderIcon from "@lucide/svelte/icons/loader"
import UserIcon from "@lucide/svelte/icons/user"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Local state for additional loaded blocks
let additionalBlocks = $state<BlockSummary[]>([])
let isLoadingMore = $state(false)

// Validator mapping
let validatorMap = $state<Map<string, Validator>>(new Map())
const avatarsVersion = $derived(getAvatarCacheVersion())

// Get validator for proposer address (already hex from API)
function getProposerValidator(proposerHex: string): Validator | undefined {
  return validatorMap.get(proposerHex.toUpperCase())
}

onMount(() => {
  const interval = setInterval(() => {
    invalidate("blocks:recent")
  }, 6_000)

  // Build validator map when validators load
  data.validators.then(async result => {
    validatorMap = await buildValidatorMap(result.validators)
    await prefetchValidatorAvatars(result.validators)
  }).catch(() => {})

  return () => clearInterval(interval)
})

// Load more older blocks using indexer
async function loadMore(currentBlocks: BlockSummary[]) {
  if (isLoadingMore || currentBlocks.length === 0) return

  isLoadingMore = true
  try {
    const allBlocks = [...currentBlocks, ...additionalBlocks]
    const oldestHeight = Math.min(...allBlocks.map(b => parseInt(b.height)))
    const moreBlocks = await indexer.blocks(data.chain.universal_chain_id, 50, oldestHeight)
    additionalBlocks = [...additionalBlocks, ...moreBlocks.map(b => ({
      height: String(b.height),
      hash: b.hash,
      time: b.time,
      proposer: b.proposer,
      txCount: b.tx_count,
      header: b.header,
      signatures: b.signatures,
      txHashes: b.tx_hashes,
    }))]
  } catch (e) {
    console.error("Failed to load more blocks:", e)
  } finally {
    isLoadingMore = false
  }
}
</script>


{#snippet blockRow(block: BlockSummary, _version: number)}
  {@const validator = getProposerValidator(block.proposer)}
  {@const avatar = getCachedAvatar(validator?.description.identity)}
  <a
    href={urls.block(block.height)}
    class="grid grid-cols-12 gap-4 px-4 py-3 border-b border-border hover:bg-muted/30 transition-colors items-center"
  >
    <div class="col-span-2">
      <span class="font-mono text-sm">#{block.height}</span>
    </div>
    <div class="col-span-4">
      <span class="font-mono text-xs text-muted-foreground">{truncateHash(block.hash, 16)}</span>
    </div>
    <div class="col-span-3">
      {#if validator}
        <div class="flex items-center gap-2">
          {#if avatar}
            <img src={avatar} alt="" class="w-5 h-5 rounded-full" />
          {:else}
            <div class="w-5 h-5 bg-muted flex items-center justify-center">
              <UserIcon class="w-3 h-3 text-muted-foreground" />
            </div>
          {/if}
          <span class="text-xs truncate">{validator.description.moniker}</span>
        </div>
      {:else}
        <span class="font-mono text-xs text-muted-foreground">{truncateAddress(formatAddress(block.proposer), 8)}</span>
      {/if}
    </div>
    <div class="col-span-1 text-center">
      <span class="text-xs font-mono px-2 py-0.5 {block.txCount > 0 ? 'bg-foreground/10' : 'text-muted-foreground'}">
        {block.txCount}
      </span>
    </div>
    <div class="col-span-2 text-right">
      <span class="text-xs text-muted-foreground">{formatTimeAgo(block.time)}</span>
    </div>
  </a>
{/snippet}

{#snippet blocksLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Recent Blocks</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(10) as _}
        <Skeleton class="h-10 w-full" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet blocksSuccess(blocks: BlockSummary[])}
  {@const combinedBlocks = [...blocks, ...additionalBlocks]}
  {@const allBlocks = combinedBlocks.filter((b, i, arr) => arr.findIndex(x => x.height === b.height) === i)}

  <div class="relative border border-border">
    <CornerMarks />

    <!-- Header -->
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium uppercase tracking-wider">Recent Blocks</span>
        <span class="text-[10px] font-mono text-muted-foreground">{allBlocks.length} blocks</span>
      </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-12 gap-4 px-4 py-2 border-b border-border bg-muted/10 text-xs font-mono uppercase tracking-wider text-muted-foreground">
      <div class="col-span-2">Height</div>
      <div class="col-span-4">Hash</div>
      <div class="col-span-3">Proposer</div>
      <div class="col-span-1 text-center">Txs</div>
      <div class="col-span-2 text-right">Time</div>
    </div>

    <!-- Rows -->
    {#each allBlocks as block (block.height)}
      {@render blockRow(block, avatarsVersion)}
    {/each}

    <!-- Load More Button -->
    <button
      onclick={() => loadMore(blocks)}
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
  </div>
{/snippet}

{#snippet blocksError(err: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6">
      <p class="text-sm font-medium text-destructive mb-2">Failed to load blocks</p>
      <p class="text-xs text-muted-foreground font-mono">{String(err)}</p>
    </div>
  </div>
{/snippet}

{@render matchPromiseWithCache(data.blocks, {
  cacheKey: `${cachePrefix}blocks:list`,
  onLoading: blocksLoading,
  onSuccess: blocksSuccess,
  onError: blocksError,
})}
