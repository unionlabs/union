<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/snippet-cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import * as Collapsible from "$lib/components/ui/collapsible/index.js"
import { formatAmount, formatTime, formatTimeAgo, truncateAddress } from "$lib/utils/format"
import { copyToClipboard as copyUtil } from "$lib/utils/clipboard"
import { fetchKeybaseAvatar } from "$lib/utils/validators.svelte"
import { urls } from "$lib/utils/urls"
import type { Validator, Delegation, StakingPool, PaginationResponse } from "$lib/types/cosmos"
import { consensusPubkeyToHexAddress, didValidatorSign } from "$lib/utils/crypto"
import ChevronDownIcon from "@lucide/svelte/icons/chevron-down"
import CopyIcon from "@lucide/svelte/icons/copy"
import ExternalLinkIcon from "@lucide/svelte/icons/external-link"
import CornerMarks from "$lib/components/corner-marks.svelte"
import { sectionHeader, dataRow } from "$lib/components/ui/snippets.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenSymbol = $derived(data.chain.assets[0]?.symbol ?? "TOKEN")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

// Keybase avatar
let avatarUrl = $state<string | null>(null)

// Validator hex address for signature matching
let validatorHexAddress = $state<string>("")

onMount(() => {
  const interval = setInterval(() => {
    invalidate("validator:data")
  }, 30_000)
  return () => clearInterval(interval)
})

function copyToClipboard(text: string) {
  copyUtil(text)
}

const getCachedPool = () => cache.get(`${cachePrefix}staking:pool`) as { pool: StakingPool } | undefined
</script>


{#snippet validatorLoading()}
  <div class="space-y-6">
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <Skeleton class="h-20 w-full" />
      </div>
    </div>
    <div class="relative border border-border">
      <CornerMarks />
      {@render sectionHeader("Details", "01")}
      <div class="p-4 space-y-3">
        {#each Array(6) as _}
          <Skeleton class="h-10" />
        {/each}
      </div>
    </div>
  </div>
{/snippet}

{#snippet validatorSuccess(result: { validator: Validator })}
  {@const validator = result.validator}
  {@const pool = getCachedPool()}
  {@const totalBonded = pool?.pool.bonded_tokens ?? "1"}
  {@const power = (BigInt(validator.tokens) * 10000n) / BigInt(totalBonded)}
  {@const commission = Number(validator.commission.commission_rates.rate) * 100}
  {@const maxCommission = Number(validator.commission.commission_rates.max_rate) * 100}
  {@const maxChangeRate = Number(validator.commission.commission_rates.max_change_rate) * 100}

  <!-- Load avatar and compute hex address -->
  {#if validator.description.identity}
    {(() => {
      fetchKeybaseAvatar(validator.description.identity).then(url => {
        avatarUrl = url
      })
      return ""
    })()}
  {/if}
  {(() => {
    consensusPubkeyToHexAddress(validator.consensus_pubkey).then(hex => {
      validatorHexAddress = hex
    })
    return ""
  })()}

  <div class="space-y-6">
    <!-- Header -->
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <div class="flex items-start gap-6">
          <!-- Avatar -->
          <div class="w-20 h-20 shrink-0 bg-muted flex items-center justify-center overflow-hidden">
            {#if avatarUrl}
              <img src={avatarUrl} alt="" class="w-20 h-20 object-cover" />
            {:else}
              <span class="text-2xl font-mono font-bold text-muted-foreground">
                {validator.description.moniker?.charAt(0).toUpperCase() || "?"}
              </span>
            {/if}
          </div>

          <!-- Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-3 mb-2">
              <h1 class="text-2xl font-bold truncate">{validator.description.moniker}</h1>
              {#if validator.jailed}
                <Badge variant="destructive">Jailed</Badge>
              {:else}
                <Badge variant="success">Active</Badge>
              {/if}
            </div>

            <div class="flex items-center gap-2 text-sm text-muted-foreground mb-4">
              <span class="font-mono">{truncateAddress(validator.operator_address, 12)}</span>
              <button onclick={() => copyToClipboard(validator.operator_address)} class="p-1 hover:bg-muted">
                <CopyIcon class="h-3 w-3" />
              </button>
            </div>

            {#if validator.description.details}
              <p class="text-sm text-muted-foreground line-clamp-2">{validator.description.details}</p>
            {/if}

            {#if validator.description.website}
              <a
                href={validator.description.website}
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground mt-2"
              >
                {validator.description.website}
                <ExternalLinkIcon class="h-3 w-3" />
              </a>
            {/if}
          </div>

          <!-- Stats -->
          <div class="grid grid-cols-2 gap-4 shrink-0">
            <div class="text-right">
              <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Voting Power</div>
              <div class="text-lg font-mono font-bold">{formatAmount(validator.tokens, tokenExponent)}</div>
              <div class="text-xs text-muted-foreground">{(Number(power) / 100).toFixed(2)}%</div>
            </div>
            <div class="text-right">
              <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-1">Commission</div>
              <div class="text-lg font-mono font-bold">{commission.toFixed(1)}%</div>
              <div class="text-xs text-muted-foreground">max {maxCommission.toFixed(0)}%</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Details Section -->
    <div class="relative border border-border">
      <CornerMarks />
      {@render sectionHeader("Validator Details", "01")}
      <div>
        {@render dataRow("Operator Address", validator.operator_address, { mono: true, copy: true })}
        {@render dataRow("Status", validator.status.replace("BOND_STATUS_", ""), { mono: true })}
        {@render dataRow("Tokens", `${formatAmount(validator.tokens, tokenExponent)} ${tokenSymbol}`, { mono: true })}
        {@render dataRow("Delegator Shares", formatAmount(validator.delegator_shares, tokenExponent), { mono: true })}
        {@render dataRow("Min Self Delegation", `${formatAmount(validator.min_self_delegation, tokenExponent)} ${tokenSymbol}`, { mono: true })}
        {#if validator.jailed}
          {@render dataRow("Unbonding Height", validator.unbonding_height, { mono: true })}
          {@render dataRow("Unbonding Time", formatTime(validator.unbonding_time), { mono: true })}
        {/if}
      </div>
    </div>

    <!-- Uptime Section -->
    {@render matchPromiseWithCache(data.recentBlocks, {
      cacheKey: `${cachePrefix}blocks:recent:full`,
      onLoading: uptimeLoading,
      onSuccess: uptimeSuccess,
      onError: uptimeError,
    })}

    <!-- Commission Section -->
    <div class="relative border border-border">
      <CornerMarks />
      {@render sectionHeader("Commission", "03")}
      <div>
        {@render dataRow("Current Rate", `${commission.toFixed(2)}%`, { mono: true })}
        {@render dataRow("Max Rate", `${maxCommission.toFixed(2)}%`, { mono: true })}
        {@render dataRow("Max Change Rate", `${maxChangeRate.toFixed(2)}% per day`, { mono: true })}
        {@render dataRow("Last Updated", formatTime(validator.commission.update_time), { mono: true })}
      </div>
    </div>

    <!-- Description Section -->
    {#if validator.description.details || validator.description.security_contact || validator.description.identity}
      <div class="relative border border-border">
        <CornerMarks />
        {@render sectionHeader("Description", "04")}
        <div>
          {#if validator.description.identity}
            {@render dataRow("Identity", validator.description.identity, { mono: true })}
          {/if}
          {#if validator.description.security_contact}
            {@render dataRow("Security Contact", validator.description.security_contact, { mono: true })}
          {/if}
          {#if validator.description.details}
            {@render dataRow("Details", validator.description.details)}
          {/if}
        </div>
      </div>
    {/if}

    <!-- Delegations Section -->
    {@render matchPromiseWithCache(data.delegations, {
      cacheKey: `${cachePrefix}validator:${data.address}:delegations`,
      onLoading: delegationsLoading,
      onSuccess: delegationsSuccess,
      onError: delegationsError,
    })}
  </div>
{/snippet}

{#snippet validatorError(_e: unknown)}
  {@render validatorLoading()}
{/snippet}

{#snippet uptimeLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Recent Blocks", "02")}
    <div class="p-4">
      <Skeleton class="h-20" />
    </div>
  </div>
{/snippet}

{#snippet uptimeSuccess(blocks: Array<{ height: string; hash: string; time: string; signatures: Array<{ validator_address: string; signature: string | null }> }>)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Recent Blocks", "02")}
    <div class="p-4">
      {#if validatorHexAddress && blocks.length > 0}
        {@const signedCount = blocks.filter(b => didValidatorSign(validatorHexAddress, b.signatures)).length}
        {@const uptimePercent = ((signedCount / blocks.length) * 100).toFixed(0)}
        <div class="flex items-end gap-px h-8 mb-4">
          {#each blocks.toReversed() as block}
            {@const signed = didValidatorSign(validatorHexAddress, block.signatures)}
            <a
              href={urls.block(block.height)}
              class="flex-1 min-w-1 transition-colors {signed ? 'h-full bg-success hover:opacity-60' : 'h-full bg-destructive hover:opacity-60'}"
              title="Block #{block.height} - {signed ? 'Signed' : 'Missed'} - {formatTimeAgo(block.time)}"
            ></a>
          {/each}
        </div>
        <div class="flex items-center justify-between">
          <div>
            <span class="text-xl font-mono font-bold">{signedCount}/{blocks.length}</span>
            <span class="text-xs text-muted-foreground ml-2">blocks signed</span>
          </div>
          <div class="text-right">
            <span class="text-xl font-mono font-bold {Number(uptimePercent) === 100 ? 'text-success' : Number(uptimePercent) > 50 ? 'text-foreground' : 'text-destructive'}">{uptimePercent}%</span>
            <span class="text-xs text-muted-foreground ml-2">uptime</span>
          </div>
        </div>
      {:else if !validatorHexAddress}
        <div class="text-sm text-muted-foreground">Loading signature data...</div>
      {:else}
        <div class="text-sm text-muted-foreground">No recent blocks available</div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet uptimeError(_e: unknown)}
  {@render uptimeLoading()}
{/snippet}

{#snippet delegationsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Delegations", "05")}
    <div class="p-4">
      <Skeleton class="h-32" />
    </div>
  </div>
{/snippet}

{#snippet delegationsSuccess(result: { delegation_responses: Delegation[]; pagination: PaginationResponse })}
  {@const delegations = result.delegation_responses}
  <Collapsible.Root>
    <div class="relative border border-border">
      <CornerMarks />
      <Collapsible.Trigger class="w-full">
        <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20 hover:bg-muted/40 transition-colors">
          <div class="flex items-center gap-3">
            <span class="text-[10px] font-mono text-muted-foreground">05</span>
            <span class="text-xs font-medium uppercase tracking-wider">Delegations</span>
            <Badge variant="default">{delegations.length}</Badge>
          </div>
          <ChevronDownIcon class="h-4 w-4 text-muted-foreground" />
        </div>
      </Collapsible.Trigger>
      <Collapsible.Content>
        {#if delegations.length > 0}
          <div class="divide-y divide-border max-h-96 overflow-y-auto">
            {#each delegations.slice(0, 50) as delegation}
              <div class="grid grid-cols-[1fr_150px] px-4 py-2.5">
                <a href={urls.account(delegation.delegation.delegator_address)} class="font-mono text-sm hover:text-foreground truncate">
                  {truncateAddress(delegation.delegation.delegator_address, 12)}
                </a>
                <span class="font-mono text-sm text-right">{formatAmount(delegation.balance.amount, tokenExponent)} {tokenSymbol}</span>
              </div>
            {/each}
            {#if delegations.length > 50}
              <div class="px-4 py-3 text-xs text-muted-foreground text-center">
                Showing 50 of {delegations.length} delegations
              </div>
            {/if}
          </div>
        {:else}
          <div class="px-4 py-8 text-center text-muted-foreground text-sm">
            No delegations found
          </div>
        {/if}
      </Collapsible.Content>
    </div>
  </Collapsible.Root>
{/snippet}

{#snippet delegationsError(_e: unknown)}
  {@render delegationsLoading()}
{/snippet}

<!-- Main Content -->
{@render matchPromiseWithCache(data.stakingPool, {
  cacheKey: `${cachePrefix}staking:pool`,
  onLoading: () => {},
  onSuccess: () => {},
  onError: () => {},
})}

{@render matchPromiseWithCache(data.validator, {
  cacheKey: `${cachePrefix}validator:${data.address}`,
  onLoading: validatorLoading,
  onSuccess: validatorSuccess,
  onError: validatorError,
})}
