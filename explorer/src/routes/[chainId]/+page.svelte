<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import { formatTimeAgo, truncateHash, formatAmount, truncateAddress } from "$lib/utils/format"
import { getMsgType, getMsgTypeVariant } from "$lib/utils/messages"
import { urls } from "$lib/utils/urls"
import type { Block, Validator, Proposal, PaginationResponse, TxResponse } from "$lib/types/cosmos"
import type { ChainStats } from "$lib/services/indexer-client"
import type { BlockSummary } from "$lib/queries/blocks"
import BoxIcon from "@lucide/svelte/icons/box"
import ArrowRightIcon from "@lucide/svelte/icons/arrow-right"
import ActivityIcon from "@lucide/svelte/icons/activity"
import UsersIcon from "@lucide/svelte/icons/users"
import CoinsIcon from "@lucide/svelte/icons/coins"
import CircleDollarSignIcon from "@lucide/svelte/icons/circle-dollar-sign"
import CheckIcon from "@lucide/svelte/icons/check"
import XIcon from "@lucide/svelte/icons/x"
import UserIcon from "@lucide/svelte/icons/user"
import CornerMarks from "$lib/components/corner-marks.svelte"
import MarketCard from "$lib/components/market-card.svelte"
import { prefetchValidatorAvatars, getCachedAvatar, getAvatarCacheVersion } from "$lib/utils/validators.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenSymbol = $derived(data.chain.assets[0]?.symbol ?? "TOKEN")
const tokenBase = $derived(data.chain.assets[0]?.base ?? "")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

// Track avatar cache version for reactivity
const avatarsVersion = $derived(getAvatarCacheVersion())

onMount(() => {
  const interval = setInterval(() => {
    invalidate("home:data")
  }, 10_000)

  // Prefetch validator avatars when validators load
  data.validators.then(async result => {
    await prefetchValidatorAvatars(result.validators)
  }).catch(() => {})

  return () => clearInterval(interval)
})

const calculateBlockTime = (blocks: BlockSummary[]) => {
  if (blocks.length < 2) return "-"
  const times = blocks.slice(0, 10).map((b) => new Date(b.time).getTime())
  const diffs = times.slice(0, -1).map((t, i) => t - times[i + 1])
  const avg = diffs.reduce((a, b) => a + b, 0) / diffs.length
  return `${(avg / 1000).toFixed(1)}s`
}

// Get cached values for immediate display (chain-specific keys)
const getCachedBlock = () => cache.get(`${cachePrefix}blocks:latest`)
const getCachedBlocks = () => cache.get(`${cachePrefix}blocks:recent`)
const getCachedValidators = () => cache.get(`${cachePrefix}validators:bonded`)
const getCachedChainStats = () => cache.get(`${cachePrefix}chain:stats`) as ChainStats | null | undefined
const getCachedProposals = () => cache.get(`${cachePrefix}proposals:all`)
</script>


{#snippet statCard(icon: typeof BoxIcon, label: string, value: string, sub?: string)}
  <div class="relative border border-border p-4 hover:bg-muted/30 transition-colors">
    <CornerMarks />
    <div class="flex items-start justify-between mb-3">
      <svelte:component this={icon} class="h-4 w-4 text-muted-foreground" />
      <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground">{label}</span>
    </div>
    <div class="text-2xl font-mono font-bold">{value}</div>
    {#if sub}
      <div class="text-xs text-muted-foreground mt-1">{sub}</div>
    {/if}
  </div>
{/snippet}

{#snippet statCardLoading()}
  <div class="relative border border-border p-4">
    <CornerMarks />
    <Skeleton class="h-4 w-16 mb-3" />
    <Skeleton class="h-8 w-24" />
  </div>
{/snippet}

<!-- Stats Section - render with cached data immediately, update when fresh -->
{#snippet statsSection()}
  {@const cachedBlock = getCachedBlock()}
  {@const cachedBlocks = getCachedBlocks()}
  {@const cachedValidators = getCachedValidators()}

  <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
    <!-- Block Height -->
    {@render matchPromiseWithCache(data.latestBlock, {
      cacheKey: `${cachePrefix}blocks:latest`,
      onLoading: statCardLoading,
      onSuccess: blockHeightCard,
      onError: statCardLoading,
    })}

    <!-- Block Time -->
    {@render matchPromiseWithCache(data.recentBlocks, {
      cacheKey: `${cachePrefix}blocks:recent`,
      onLoading: statCardLoading,
      onSuccess: blockTimeCard,
      onError: statCardLoading,
    })}

    <!-- Validators -->
    {@render matchPromiseWithCache(data.validators, {
      cacheKey: `${cachePrefix}validators:bonded`,
      onLoading: statCardLoading,
      onSuccess: validatorsCard,
      onError: statCardLoading,
    })}

    <!-- Bonded -->
    {@render matchPromiseWithCache(data.chainStats, {
      cacheKey: `${cachePrefix}chain:stats`,
      onLoading: statCardLoading,
      onSuccess: bondedCard,
      onError: statCardLoading,
    })}

    <!-- Total Supply -->
    {@render matchPromiseWithCache(data.chainStats, {
      cacheKey: `${cachePrefix}chain:stats`,
      onLoading: statCardLoading,
      onSuccess: supplyCard,
      onError: statCardLoading,
    })}
  </div>
{/snippet}

{#snippet blockHeightCard(block: Block)}
  {@render statCard(BoxIcon, "Latest Block", `#${block.block.header.height}`, formatTimeAgo(block.block.header.time))}
{/snippet}

{#snippet blockTimeCard(blocks: BlockSummary[])}
  {@render statCard(ActivityIcon, "Block Time", calculateBlockTime(blocks), "avg last 10 blocks")}
{/snippet}

{#snippet validatorsCard(result: { validators: Validator[]; pagination: PaginationResponse })}
  {@const activeValidators = result.validators.filter(v => !v.jailed)}
  {@render statCard(UsersIcon, "Validators", String(activeValidators.length), `${result.validators.length} total`)}
{/snippet}

{#snippet bondedCard(stats: ChainStats | null)}
  {@render statCard(CoinsIcon, "Bonded", stats ? formatAmount(stats.bonded_tokens, tokenExponent) : "-", `${tokenSymbol} staked`)}
{/snippet}

{#snippet supplyCard(stats: ChainStats | null)}
  {@render statCard(CircleDollarSignIcon, "Supply", stats ? formatAmount(stats.total_supply, tokenExponent) : "-", `${tokenSymbol} total`)}
{/snippet}

<!-- Blocks Section -->
{#snippet blocksLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Recent Blocks</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(6) as _}
        <Skeleton class="h-10" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet blocksSuccess(blocks: BlockSummary[])}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Recent Blocks</span>
      <a href={urls.blocks()} class="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1">
        View all <ArrowRightIcon class="h-3 w-3" />
      </a>
    </div>
    <div class="divide-y divide-border">
      {#each blocks.slice(0, 6) as b}
        <a href={urls.block(b.height)} class="flex items-center justify-between px-4 py-2.5 hover:bg-muted/30 transition-colors">
          <div class="flex items-center gap-3">
            <span class="font-mono text-sm">#{b.height}</span>
            <span class="font-mono text-xs text-muted-foreground">{truncateHash(b.hash, 8)}</span>
          </div>
          <div class="flex items-center gap-4">
            <span class="text-xs font-mono px-1.5 py-0.5 {b.txCount > 0 ? 'bg-foreground/10' : 'text-muted-foreground'}">
              {b.txCount} txs
            </span>
            <span class="text-xs text-muted-foreground w-16 text-right">{formatTimeAgo(b.time)}</span>
          </div>
        </a>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet blocksError(_e: unknown)}
  {@render blocksLoading()}
{/snippet}

<!-- Validators Section -->
{#snippet validatorsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Top Validators</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(6) as _}
        <Skeleton class="h-10" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet validatorsSuccess(result: { validators: Validator[]; pagination: PaginationResponse })}
  {@const cachedStats = getCachedChainStats()}
  {@const sortedValidators = [...result.validators].sort((a, b) => Number(BigInt(b.tokens) - BigInt(a.tokens)))}
  {@const _ = avatarsVersion}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Top Validators</span>
      <a href={urls.validators()} class="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1">
        View all <ArrowRightIcon class="h-3 w-3" />
      </a>
    </div>
    <div class="divide-y divide-border">
      {#each sortedValidators.slice(0, 6) as validator, i}
        {@const totalBonded = cachedStats?.bonded_tokens ?? "1"}
        {@const power = (BigInt(validator.tokens) * 10000n) / BigInt(totalBonded)}
        {@const avatar = getCachedAvatar(validator.description.identity)}
        <a href={urls.validator(validator.operator_address)} class="flex items-center justify-between px-4 py-2.5 hover:bg-muted/30 transition-colors">
          <div class="flex items-center gap-3">
            <span class="text-xs font-mono text-muted-foreground w-5">{String(i + 1).padStart(2, '0')}</span>
            {#if avatar}
              <img src={avatar} alt="" class="w-5 h-5 rounded-full" />
            {:else}
              <div class="w-5 h-5 bg-muted flex items-center justify-center">
                <UserIcon class="w-3 h-3 text-muted-foreground" />
              </div>
            {/if}
            <span class="text-sm truncate max-w-[150px]">{validator.description.moniker}</span>
            {#if validator.jailed}
              <Badge variant="destructive">Jailed</Badge>
            {/if}
          </div>
          <div class="flex items-center gap-4">
            <span class="text-xs font-mono">{(Number(power) / 100).toFixed(2)}%</span>
            <span class="text-xs text-muted-foreground w-12 text-right">
              {(Number(validator.commission.commission_rates.rate) * 100).toFixed(0)}%
            </span>
          </div>
        </a>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet validatorsError(_e: unknown)}
  {@render validatorsLoading()}
{/snippet}

<!-- Governance Section -->
{#snippet governanceLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Governance</span>
    </div>
    <div class="p-4">
      <Skeleton class="h-24" />
    </div>
  </div>
{/snippet}

{#snippet governanceSuccess(result: { proposals: Proposal[]; pagination: PaginationResponse })}
  {@const proposals = result.proposals}
  {@const votingProposals = proposals.filter(p => p.status === "PROPOSAL_STATUS_VOTING_PERIOD")}
  {#if proposals.length > 0}
    <div class="relative border border-border">
      <CornerMarks />
      <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
        <div class="flex items-center gap-3">
          <span class="text-xs font-medium uppercase tracking-wider">Governance</span>
          {#if votingProposals.length > 0}
            <Badge variant="success">{votingProposals.length} voting</Badge>
          {/if}
        </div>
        <a href={urls.governance()} class="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1">
          View all <ArrowRightIcon class="h-3 w-3" />
        </a>
      </div>
      <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-px bg-border">
        {#each proposals.slice(0, 3) as proposal}
          {@const isVoting = proposal.status === "PROPOSAL_STATUS_VOTING_PERIOD"}
          <a href={urls.proposal(proposal.id)} class="bg-background p-4 hover:bg-muted/30 transition-colors">
            <div class="flex items-center gap-2 mb-2">
              <span class="text-xs font-mono text-muted-foreground">#{proposal.id}</span>
              <Badge variant={isVoting ? "success" : "secondary"}>
                {isVoting ? "Voting" : proposal.status.replace("PROPOSAL_STATUS_", "").toLowerCase()}
              </Badge>
            </div>
            <p class="text-sm font-medium line-clamp-2">{proposal.title}</p>
          </a>
        {/each}
      </div>
    </div>
  {/if}
{/snippet}

{#snippet governanceError(_e: unknown)}
  <!-- Just hide on error -->
{/snippet}

<!-- Recent Transactions Section -->
{#snippet txsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Recent Transactions</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(5) as _}
        <Skeleton class="h-12" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet txsSuccess(result: { tx_responses: TxResponse[]; pagination: PaginationResponse })}
  {@const txs = result.tx_responses ?? []}
  {#if txs.length > 0}
    <div class="relative border border-border">
      <CornerMarks />
      <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
        <span class="text-xs font-medium uppercase tracking-wider">Recent Transactions</span>
        <a href={urls.transactions()} class="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1">
          View all <ArrowRightIcon class="h-3 w-3" />
        </a>
      </div>
      <div class="divide-y divide-border">
        {#each txs.slice(0, 6) as tx}
          {@const msg = tx.tx.body.messages[0] as { "@type": string }}
          {@const msgType = getMsgType(msg)}
          <a href={urls.transaction(tx.txhash)} class="flex items-center justify-between px-4 py-2.5 hover:bg-muted/30 transition-colors">
            <div class="flex items-center gap-3">
              <div class="w-5 h-5 flex items-center justify-center {tx.code === 0 ? 'bg-success/20' : 'bg-destructive/20'}">
                {#if tx.code === 0}
                  <CheckIcon class="h-3 w-3 text-success" />
                {:else}
                  <XIcon class="h-3 w-3 text-destructive" />
                {/if}
              </div>
              <span class="font-mono text-sm">{truncateHash(tx.txhash, 8)}</span>
              <Badge variant={getMsgTypeVariant(msgType)}>{msgType}</Badge>
            </div>
            <div class="flex items-center gap-4">
              <span class="text-xs font-mono text-muted-foreground">#{tx.height}</span>
              <span class="text-xs text-muted-foreground w-16 text-right">{formatTimeAgo(tx.timestamp)}</span>
            </div>
          </a>
        {/each}
      </div>
    </div>
  {/if}
{/snippet}

{#snippet txsError(_e: unknown)}
  <!-- Just hide on error -->
{/snippet}

<!-- Chain Info Section -->
{#snippet chainInfoLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Chain Info</span>
    </div>
    <div class="p-4">
      <Skeleton class="h-16" />
    </div>
  </div>
{/snippet}

{#snippet chainInfoSuccess(block: Block)}
  {@const cachedValidators = getCachedValidators()}
  {@const cachedProposals = getCachedProposals()}
  {@const activeValidators = cachedValidators?.validators.filter(v => !v.jailed) ?? []}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Chain Info</span>
    </div>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-px bg-border">
      <div class="bg-background p-4">
        <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground block mb-1">Chain ID</span>
        <span class="text-sm font-mono">{block.block.header.chain_id}</span>
      </div>
      <div class="bg-background p-4">
        <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground block mb-1">Height</span>
        <span class="text-sm font-mono">{block.block.header.height}</span>
      </div>
      <div class="bg-background p-4">
        <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground block mb-1">Validators</span>
        <span class="text-sm font-mono">{activeValidators.length} / {cachedValidators?.validators.length ?? "-"}</span>
      </div>
      <div class="bg-background p-4">
        <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground block mb-1">Proposals</span>
        <span class="text-sm font-mono">{cachedProposals?.proposals.length ?? "-"}</span>
      </div>
    </div>
  </div>
{/snippet}

{#snippet chainInfoError(_e: unknown)}
  {@render chainInfoLoading()}
{/snippet}

<div class="space-y-6">
  <!-- Stats - each card cached individually -->
  {@render statsSection()}

  <!-- Market Card -->
  <MarketCard />

  <!-- Main Grid -->
  <div class="grid lg:grid-cols-2 gap-6">
    {@render matchPromiseWithCache(data.recentBlocks, {
      cacheKey: `${cachePrefix}blocks:recent`,
      onLoading: blocksLoading,
      onSuccess: blocksSuccess,
      onError: blocksError,
    })}

    {@render matchPromiseWithCache(data.validators, {
      cacheKey: `${cachePrefix}validators:bonded`,
      onLoading: validatorsLoading,
      onSuccess: validatorsSuccess,
      onError: validatorsError,
    })}
  </div>

  <!-- Recent Transactions -->
  {@render matchPromiseWithCache(data.recentTxs, {
    cacheKey: `${cachePrefix}txs:recent`,
    onLoading: txsLoading,
    onSuccess: txsSuccess,
    onError: txsError,
  })}

  <!-- Governance -->
  {@render matchPromiseWithCache(data.proposals, {
    cacheKey: `${cachePrefix}proposals:all`,
    onLoading: governanceLoading,
    onSuccess: governanceSuccess,
    onError: governanceError,
  })}

  <!-- Chain Info -->
  {@render matchPromiseWithCache(data.latestBlock, {
    cacheKey: `${cachePrefix}blocks:latest`,
    onLoading: chainInfoLoading,
    onSuccess: chainInfoSuccess,
    onError: chainInfoError,
  })}
</div>
