<script lang="ts">
import { matchPromiseWithCache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge } from "$lib/components/ui/badge/index.js"
import { formatDuration, formatParamPercent, formatParamNumber, formatBool, formatParamKey } from "$lib/utils/params"
import { formatAmount } from "$lib/utils/format"
import type { StakingParams, SlashingParams, DistributionParams, GovParams, MintParams, Coin } from "$lib/types/cosmos"
import CoinsIcon from "@lucide/svelte/icons/coins"
import ShieldIcon from "@lucide/svelte/icons/shield"
import ScaleIcon from "@lucide/svelte/icons/scale"
import VoteIcon from "@lucide/svelte/icons/vote"
import TrendingUpIcon from "@lucide/svelte/icons/trending-up"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

type ParamValue = string | number | boolean | Coin[] | null | undefined

// Format a param value based on its key and type
function formatValue(key: string, value: ParamValue): string {
  if (value === null || value === undefined) return "-"

  // Handle coin arrays (deposits)
  if (Array.isArray(value)) {
    return value.map(c => `${formatAmount(c.amount, tokenExponent)} ${c.denom}`).join(", ") || "-"
  }

  // Handle booleans
  if (typeof value === "boolean") {
    return formatBool(value)
  }

  const strValue = String(value)
  const keyLower = key.toLowerCase()

  // Duration fields
  if (keyLower.includes("period") || keyLower.includes("time") || keyLower.includes("duration") || keyLower.includes("window")) {
    return formatDuration(strValue)
  }

  // Percentage/ratio fields
  if (keyLower.includes("rate") || keyLower.includes("ratio") || keyLower.includes("tax") ||
      keyLower.includes("quorum") || keyLower.includes("threshold") || keyLower.includes("fraction") ||
      keyLower.includes("reward") || keyLower.includes("inflation") || keyLower.includes("bonded")) {
    return formatParamPercent(strValue)
  }

  // Number fields
  if (keyLower.includes("max") || keyLower.includes("entries") || keyLower.includes("blocks") || keyLower.includes("validators")) {
    return formatParamNumber(strValue)
  }

  return strValue
}

// Get badge variant based on value type
function getValueVariant(key: string, value: ParamValue): "default" | "success" | "warning" | "info" | "secondary" {
  if (typeof value === "boolean") {
    return value ? "success" : "secondary"
  }

  const keyLower = key.toLowerCase()
  if (keyLower.includes("slash") || keyLower.includes("jail") || keyLower.includes("burn") || keyLower.includes("veto")) {
    return "warning"
  }
  if (keyLower.includes("reward") || keyLower.includes("bonus")) {
    return "success"
  }

  return "default"
}
</script>

{#snippet sectionHeader(icon: typeof SettingsIcon, title: string, num: string)}
  <div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-muted/20">
    <span class="text-[10px] font-mono text-muted-foreground">{num}</span>
    <svelte:component this={icon} class="h-4 w-4 text-muted-foreground" />
    <span class="text-xs font-medium uppercase tracking-wider">{title}</span>
  </div>
{/snippet}

{#snippet paramRow(key: string, value: ParamValue, options?: { highlight?: boolean })}
  {@const formattedValue = formatValue(key, value)}
  {@const variant = getValueVariant(key, value)}
  <div class="flex items-center justify-between py-2.5 px-4 hover:bg-muted/30 border-b border-border last:border-b-0 {options?.highlight ? 'bg-muted/10' : ''}">
    <span class="text-xs text-muted-foreground">{formatParamKey(key)}</span>
    <Badge variant={variant}>{formattedValue}</Badge>
  </div>
{/snippet}

{#snippet stakingSection(params: StakingParams)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(CoinsIcon, "Staking", "01")}
    {@render paramRow("unbonding_time", params.unbonding_time, { highlight: true })}
    {@render paramRow("max_validators", params.max_validators)}
    {@render paramRow("max_entries", params.max_entries)}
    {@render paramRow("historical_entries", params.historical_entries)}
    {@render paramRow("bond_denom", params.bond_denom)}
    {@render paramRow("min_commission_rate", params.min_commission_rate)}
  </div>
{/snippet}

{#snippet slashingSection(params: SlashingParams)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(ShieldIcon, "Slashing", "02")}
    {@render paramRow("signed_blocks_window", params.signed_blocks_window)}
    {@render paramRow("min_signed_per_window", params.min_signed_per_window, { highlight: true })}
    {@render paramRow("downtime_jail_duration", params.downtime_jail_duration)}
    {@render paramRow("slash_fraction_double_sign", params.slash_fraction_double_sign, { highlight: true })}
    {@render paramRow("slash_fraction_downtime", params.slash_fraction_downtime)}
  </div>
{/snippet}

{#snippet distributionSection(params: DistributionParams)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(ScaleIcon, "Distribution", "03")}
    {@render paramRow("community_tax", params.community_tax, { highlight: true })}
    {@render paramRow("base_proposer_reward", params.base_proposer_reward)}
    {@render paramRow("bonus_proposer_reward", params.bonus_proposer_reward)}
    {@render paramRow("withdraw_addr_enabled", params.withdraw_addr_enabled)}
  </div>
{/snippet}

{#snippet govSection(params: GovParams)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(VoteIcon, "Governance", "04")}
    {@render paramRow("min_deposit", params.min_deposit, { highlight: true })}
    {@render paramRow("max_deposit_period", params.max_deposit_period)}
    {@render paramRow("voting_period", params.voting_period, { highlight: true })}
    {@render paramRow("quorum", params.quorum)}
    {@render paramRow("threshold", params.threshold)}
    {@render paramRow("veto_threshold", params.veto_threshold)}
    {@render paramRow("min_initial_deposit_ratio", params.min_initial_deposit_ratio)}
    {@render paramRow("expedited_voting_period", params.expedited_voting_period)}
    {@render paramRow("expedited_threshold", params.expedited_threshold)}
    {@render paramRow("expedited_min_deposit", params.expedited_min_deposit)}
    {@render paramRow("burn_vote_quorum", params.burn_vote_quorum)}
    {@render paramRow("burn_proposal_deposit_prevote", params.burn_proposal_deposit_prevote)}
    {@render paramRow("burn_vote_veto", params.burn_vote_veto)}
  </div>
{/snippet}

{#snippet mintSection(params: MintParams)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(TrendingUpIcon, "Mint", "05")}
    {@render paramRow("mint_denom", params.mint_denom)}
    {@render paramRow("inflation_rate_change", params.inflation_rate_change)}
    {@render paramRow("inflation_max", params.inflation_max, { highlight: true })}
    {@render paramRow("inflation_min", params.inflation_min)}
    {@render paramRow("goal_bonded", params.goal_bonded)}
    {@render paramRow("blocks_per_year", params.blocks_per_year)}
  </div>
{/snippet}

{#snippet loading()}
  <div class="grid gap-6 lg:grid-cols-2">
    {#each Array(6) as _}
      <div class="relative border border-border">
        <CornerMarks />
        <div class="px-4 py-3 border-b border-border bg-muted/20">
          <Skeleton class="h-4 w-24" />
        </div>
        <div class="p-4 space-y-3">
          {#each Array(5) as _}
            <Skeleton class="h-8" />
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet success(result: {
  staking: StakingParams | null
  slashing: SlashingParams | null
  distribution: DistributionParams | null
  gov: GovParams | null
  mint: MintParams | null
})}
  <div class="grid gap-6 lg:grid-cols-2">
    {#if result.staking}
      {@render stakingSection(result.staking)}
    {/if}

    {#if result.slashing}
      {@render slashingSection(result.slashing)}
    {/if}

    {#if result.distribution}
      {@render distributionSection(result.distribution)}
    {/if}

    {#if result.gov}
      {@render govSection(result.gov)}
    {/if}

    {#if result.mint}
      {@render mintSection(result.mint)}
    {/if}
  </div>
{/snippet}

{#snippet error(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-12 text-center text-muted-foreground text-sm">
      Failed to load parameters
    </div>
  </div>
{/snippet}

{@render matchPromiseWithCache(data.params, {
  cacheKey: `${cachePrefix}params:all`,
  onLoading: loading,
  onSuccess: success,
  onError: error,
})}
