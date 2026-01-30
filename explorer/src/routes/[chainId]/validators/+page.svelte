<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import { formatAmount, formatPercent } from "$lib/utils/format"
import { prefetchValidatorAvatars, getCachedAvatar } from "$lib/utils/validators.svelte"
import { urls } from "$lib/utils/urls"
import type { Validator, StakingPool, PaginationResponse } from "$lib/types/cosmos"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenSymbol = $derived(data.chain.assets[0]?.symbol ?? "TOKEN")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

let avatarsVersion = $state(0)

onMount(() => {
  const interval = setInterval(() => {
    invalidate("validators:data")
  }, 30_000)
  return () => clearInterval(interval)
})

async function loadAvatars(validators: Validator[]) {
  await prefetchValidatorAvatars(validators)
  avatarsVersion++
}

function getAvatarUrl(identity?: string): string | null {
  return getCachedAvatar(identity)
}

// Get cached values with chain-specific keys
const getCachedPool = () => cache.get(`${cachePrefix}staking:pool`) as { pool: StakingPool } | undefined
const getCachedValidators = () => cache.get(`${cachePrefix}validators:bonded`) as { validators: Validator[] } | undefined
</script>


{#snippet validatorsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Validators</span>
    </div>
    <div class="p-4 space-y-2">
      {#each Array(10) as _}
        <Skeleton class="h-14" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet validatorsSuccess(result: { validators: Validator[]; pagination: PaginationResponse })}
  {@const pool = getCachedPool()}
  {@const totalBonded = pool?.pool.bonded_tokens ?? "1"}
  {@const sortedValidators = [...result.validators].sort((a, b) => Number(BigInt(b.tokens) - BigInt(a.tokens)))}

  <!-- Load avatars when validators arrive -->
  {(() => { loadAvatars(sortedValidators); return ""; })()}

  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center gap-3">
        <span class="text-xs font-medium uppercase tracking-wider">Active Validators</span>
<Badge variant="default">
          {result.validators.filter(v => !v.jailed).length} / {result.validators.length}
        </Badge>
      </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-[3rem_1fr_120px_100px_80px] gap-4 px-4 py-2 border-b border-border bg-muted/10 text-[10px] font-mono uppercase tracking-wider text-muted-foreground">
      <div>Rank</div>
      <div>Validator</div>
      <div class="text-right">Voting Power</div>
      <div class="text-right">Commission</div>
      <div class="text-right">Status</div>
    </div>

    <!-- Validators List -->
    <div class="divide-y divide-border">
      {#each sortedValidators as validator, i}
        {@const power = (BigInt(validator.tokens) * 10000n) / BigInt(totalBonded)}
        {@const commission = Number(validator.commission.commission_rates.rate) * 100}
        {@const avatarUrl = getAvatarUrl(validator.description.identity)}
        <a
          href={urls.validator(validator.operator_address)}
          class="grid grid-cols-[3rem_1fr_120px_100px_80px] gap-4 px-4 py-3 hover:bg-muted/30 transition-colors items-center"
        >
          <!-- Rank -->
          <div class="text-xs font-mono text-muted-foreground">
            {String(i + 1).padStart(2, "0")}
          </div>

          <!-- Validator Info -->
          <div class="flex items-center gap-3 min-w-0">
            <div class="w-8 h-8 shrink-0 bg-muted flex items-center justify-center overflow-hidden">
              {#if avatarUrl}
                <img src={avatarUrl} alt="" class="w-8 h-8 object-cover" />
              {:else}
                <span class="text-xs font-mono font-bold text-muted-foreground">
                  {validator.description.moniker?.charAt(0).toUpperCase() || "?"}
                </span>
              {/if}
            </div>
            <div class="min-w-0">
              <div class="text-sm font-medium truncate">{validator.description.moniker}</div>
              {#if validator.description.website}
                <div class="text-[10px] text-muted-foreground truncate">{validator.description.website}</div>
              {/if}
            </div>
          </div>

          <!-- Voting Power -->
          <div class="text-right">
            <div class="text-xs font-mono">{formatAmount(validator.tokens, tokenExponent)}</div>
            <div class="text-[10px] text-muted-foreground">{(Number(power) / 100).toFixed(2)}%</div>
          </div>

          <!-- Commission -->
          <div class="text-right text-xs font-mono">
            {commission.toFixed(1)}%
          </div>

          <!-- Status -->
          <div class="text-right">
            {#if validator.jailed}
              <Badge variant="destructive">Jailed</Badge>
            {:else}
              <Badge variant="success">Active</Badge>
            {/if}
          </div>
        </a>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet validatorsError(_e: unknown)}
  {@render validatorsLoading()}
{/snippet}

{#snippet statsSection()}
  {@const pool = getCachedPool()}
  {@const cachedValidators = getCachedValidators()}
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
    <!-- Total Bonded -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Total Bonded</div>
      {#if pool}
        <div class="text-xl font-mono font-bold">{formatAmount(pool.pool.bonded_tokens, tokenExponent)}</div>
        <div class="text-xs text-muted-foreground">{tokenSymbol} staked</div>
      {:else}
        <Skeleton class="h-7 w-24" />
      {/if}
    </div>

    <!-- Not Bonded -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Not Bonded</div>
      {#if pool}
        <div class="text-xl font-mono font-bold">{formatAmount(pool.pool.not_bonded_tokens, tokenExponent)}</div>
        <div class="text-xs text-muted-foreground">{tokenSymbol} unbonding</div>
      {:else}
        <Skeleton class="h-7 w-24" />
      {/if}
    </div>

    <!-- Bonded Ratio -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Bonded Ratio</div>
      {#if pool}
        {@const bonded = BigInt(pool.pool.bonded_tokens)}
        {@const notBonded = BigInt(pool.pool.not_bonded_tokens)}
        {@const total = bonded + notBonded}
        {@const ratio = total > 0n ? (bonded * 10000n) / total : 0n}
        <div class="text-xl font-mono font-bold">{(Number(ratio) / 100).toFixed(2)}%</div>
        <div class="text-xs text-muted-foreground">of supply</div>
      {:else}
        <Skeleton class="h-7 w-24" />
      {/if}
    </div>

    <!-- Active Validators - from cache -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Active Validators</div>
      {#if cachedValidators}
        {@const active = cachedValidators.validators.filter(v => !v.jailed)}
        <div class="text-xl font-mono font-bold">{active.length}</div>
        <div class="text-xs text-muted-foreground">{cachedValidators.validators.length} total</div>
      {:else}
        <Skeleton class="h-7 w-24" />
      {/if}
    </div>
  </div>
{/snippet}

<div class="space-y-6">
  <!-- Stats Section - uses pool cache -->
  {@render matchPromiseWithCache(data.stakingPool, {
    cacheKey: `${cachePrefix}staking:pool`,
    onLoading: () => {},
    onSuccess: () => {},
    onError: () => {},
  })}
  {@render statsSection()}

  <!-- Validators Table -->
  {@render matchPromiseWithCache(data.validators, {
    cacheKey: `${cachePrefix}validators:bonded`,
    onLoading: validatorsLoading,
    onSuccess: validatorsSuccess,
    onError: validatorsError,
  })}
</div>
