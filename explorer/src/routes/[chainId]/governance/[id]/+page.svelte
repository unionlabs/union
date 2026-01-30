<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge, type BadgeVariant } from "$lib/components/ui/badge/index.js"
import { formatAmount, formatTime, formatTimeAgo, truncateAddress } from "$lib/utils/format"
import { urls } from "$lib/utils/urls"
import type { Proposal, StakingPool, Validator, PaginationResponse } from "$lib/types/cosmos"
import VoteIcon from "@lucide/svelte/icons/vote"
import CheckCircleIcon from "@lucide/svelte/icons/check-circle"
import XCircleIcon from "@lucide/svelte/icons/x-circle"
import ClockIcon from "@lucide/svelte/icons/clock"
import AlertCircleIcon from "@lucide/svelte/icons/alert-circle"
import CalendarIcon from "@lucide/svelte/icons/calendar"
import UsersIcon from "@lucide/svelte/icons/users"
import CoinsIcon from "@lucide/svelte/icons/coins"
import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left"
import ExternalLinkIcon from "@lucide/svelte/icons/external-link"
import CornerMarks from "$lib/components/corner-marks.svelte"
import { sectionHeader } from "$lib/components/ui/snippets.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenSymbol = $derived(data.chain.assets[0]?.symbol ?? "TOKEN")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

onMount(() => {
  const interval = setInterval(() => {
    invalidate("proposal:data")
  }, 15_000)
  return () => clearInterval(interval)
})

// Status mapping
const getStatusInfo = (status: string): { label: string; variant: BadgeVariant; icon: typeof VoteIcon } => {
  const s = status.toUpperCase()
  if (s.includes("VOTING")) return { label: "Voting", variant: "info", icon: VoteIcon }
  if (s.includes("PASSED")) return { label: "Passed", variant: "success", icon: CheckCircleIcon }
  if (s.includes("REJECTED")) return { label: "Rejected", variant: "destructive", icon: XCircleIcon }
  if (s.includes("DEPOSIT")) return { label: "Deposit Period", variant: "warning", icon: ClockIcon }
  if (s.includes("FAILED")) return { label: "Failed", variant: "destructive", icon: AlertCircleIcon }
  return { label: status.replace("PROPOSAL_STATUS_", ""), variant: "secondary", icon: AlertCircleIcon }
}

// Get proposal type from messages
const getProposalType = (proposal: Proposal): string => {
  if (proposal.messages.length === 0) return "Text Proposal"
  const type = proposal.messages[0]["@type"] ?? ""
  if (type.includes("SoftwareUpgrade")) return "Software Upgrade"
  if (type.includes("ParameterChange") || type.includes("MsgUpdateParams")) return "Parameter Change"
  if (type.includes("CommunityPool")) return "Community Spend"
  if (type.includes("CancelUpgrade")) return "Cancel Upgrade"
  if (type.includes("ClientUpdate")) return "IBC Client Update"
  return type.split(".").pop()?.replace("Msg", "") ?? "Text Proposal"
}

// Time remaining for voting
const getTimeRemaining = (endTime: string): { text: string; urgent: boolean } => {
  const end = new Date(endTime)
  const now = new Date()
  const diff = end.getTime() - now.getTime()

  if (diff <= 0) return { text: "Ended", urgent: false }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const mins = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))

  if (days > 0) return { text: `${days}d ${hours}h remaining`, urgent: days < 2 }
  if (hours > 0) return { text: `${hours}h ${mins}m remaining`, urgent: true }
  return { text: `${mins}m remaining`, urgent: true }
}

// Get validator name from cache
const getValidatorName = (voterAddress: string): string | null => {
  const cached = cache.get("validators:bonded") as { validators: Validator[] } | undefined
  if (!cached) return null
  // Voters use account addresses, not operator addresses, so this might not match
  // But we can try to find by delegator if available
  return null
}

// Vote option to label
const getVoteLabel = (option: string): { label: string; color: string } => {
  const o = option.toUpperCase()
  if (o.includes("YES") && !o.includes("VETO")) return { label: "Yes", color: "text-green-500" }
  if (o.includes("NO") && o.includes("VETO")) return { label: "No with Veto", color: "text-orange-500" }
  if (o.includes("NO")) return { label: "No", color: "text-red-500" }
  if (o.includes("ABSTAIN")) return { label: "Abstain", color: "text-gray-400" }
  return { label: option, color: "text-muted-foreground" }
}

const getCachedPool = () => cache.get("staking:pool") as { pool: StakingPool } | undefined
</script>


{#snippet proposalLoading()}
  <div class="space-y-6">
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <Skeleton class="h-32" />
      </div>
    </div>
    <div class="grid grid-cols-2 gap-4">
      <div class="relative border border-border p-4">
        <CornerMarks />
        <Skeleton class="h-24" />
      </div>
      <div class="relative border border-border p-4">
        <CornerMarks />
        <Skeleton class="h-24" />
      </div>
    </div>
  </div>
{/snippet}

{#snippet proposalSuccess(result: { proposal: Proposal })}
  {@const proposal = result.proposal}
  {@const status = getStatusInfo(proposal.status)}
  {@const type = getProposalType(proposal)}
  {@const isVoting = proposal.status.toUpperCase().includes("VOTING")}
  {@const timeRemaining = isVoting ? getTimeRemaining(proposal.voting_end_time) : null}

  <!-- Header -->
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6">
      <!-- Back link -->
      <a href={urls.governance()} class="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground mb-4 transition-colors">
        <ArrowLeftIcon class="h-3 w-3" />
        Back to Proposals
      </a>

      <div class="flex items-start justify-between gap-4">
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-2">
            <span class="text-[10px] font-mono text-muted-foreground">#{proposal.id}</span>
            <Badge variant="default">{type}</Badge>
            <Badge variant={status.variant}>{status.label}</Badge>
          </div>
          <h1 class="text-xl font-bold mb-2">{proposal.title || "Untitled Proposal"}</h1>
          {#if proposal.summary}
            <p class="text-sm text-muted-foreground">{proposal.summary}</p>
          {/if}
        </div>

        {#if isVoting && timeRemaining}
          <div class="text-right shrink-0 p-4 border border-border bg-muted/20">
            <div class="text-[10px] font-mono uppercase text-muted-foreground mb-1">Voting Ends</div>
            <div class="text-lg font-mono font-bold {timeRemaining.urgent ? 'text-amber-500' : 'text-blue-500'}">{timeRemaining.text}</div>
            <div class="text-[10px] text-muted-foreground mt-1">{formatTime(proposal.voting_end_time)}</div>
          </div>
        {/if}
      </div>

      {#if proposal.proposer}
        <div class="mt-4 pt-4 border-t border-border">
          <span class="text-[10px] font-mono uppercase text-muted-foreground">Proposer: </span>
          <a href={urls.account(proposal.proposer)} class="text-xs font-mono hover:underline">{truncateAddress(proposal.proposer, 12)}</a>
        </div>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet proposalError(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6 text-center">
      <AlertCircleIcon class="h-8 w-8 text-muted-foreground mx-auto mb-2" />
      <p class="text-sm text-muted-foreground">Proposal not found</p>
      <a href={urls.governance()} class="text-xs text-blue-500 hover:underline mt-2 inline-block">Back to proposals</a>
    </div>
  </div>
{/snippet}

{#snippet tallySuccess(result: { tally: { yes_count: string; abstain_count: string; no_count: string; no_with_veto_count: string } })}
  {@const tally = result.tally}
  {@const yes = BigInt(tally.yes_count || "0")}
  {@const no = BigInt(tally.no_count || "0")}
  {@const abstain = BigInt(tally.abstain_count || "0")}
  {@const veto = BigInt(tally.no_with_veto_count || "0")}
  {@const total = yes + no + abstain + veto}
  {@const pool = getCachedPool()}
  {@const bonded = pool ? BigInt(pool.pool.bonded_tokens) : 0n}
  {@const turnout = bonded > 0n ? Number((total * 10000n) / bonded) / 100 : 0}

  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Voting Results")}

    <div class="p-4 space-y-4">
      <!-- Turnout -->
      {#if pool}
        <div class="flex items-center justify-between text-sm mb-2">
          <span class="text-muted-foreground">Turnout</span>
          <span class="font-mono font-medium">{turnout.toFixed(2)}%</span>
        </div>
      {/if}

      <!-- Main tally bar -->
      <div class="h-4 w-full overflow-hidden bg-muted/50 rounded flex">
        {#if total > 0n}
          {@const yesP = Number((yes * 10000n) / total) / 100}
          {@const noP = Number((no * 10000n) / total) / 100}
          {@const vetoP = Number((veto * 10000n) / total) / 100}
          {@const abstainP = Number((abstain * 10000n) / total) / 100}
          <div class="bg-green-500 h-full transition-all" style="width: {yesP}%"></div>
          <div class="bg-red-500 h-full transition-all" style="width: {noP}%"></div>
          <div class="bg-orange-500 h-full transition-all" style="width: {vetoP}%"></div>
          <div class="bg-gray-400 h-full transition-all" style="width: {abstainP}%"></div>
        {:else}
          <div class="text-xs text-muted-foreground flex items-center justify-center w-full">No votes yet</div>
        {/if}
      </div>

      <!-- Vote breakdown -->
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 pt-2">
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 bg-green-500 rounded-sm"></span>
            <span class="text-xs font-medium">Yes</span>
          </div>
          <div class="text-lg font-mono font-bold text-green-500">
            {total > 0n ? (Number((yes * 10000n) / total) / 100).toFixed(2) : "0.00"}%
          </div>
          <div class="text-[10px] text-muted-foreground font-mono">{formatAmount(yes.toString(), tokenExponent)} {tokenSymbol}</div>
        </div>

        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 bg-red-500 rounded-sm"></span>
            <span class="text-xs font-medium">No</span>
          </div>
          <div class="text-lg font-mono font-bold text-red-500">
            {total > 0n ? (Number((no * 10000n) / total) / 100).toFixed(2) : "0.00"}%
          </div>
          <div class="text-[10px] text-muted-foreground font-mono">{formatAmount(no.toString(), tokenExponent)} {tokenSymbol}</div>
        </div>

        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 bg-orange-500 rounded-sm"></span>
            <span class="text-xs font-medium">No with Veto</span>
          </div>
          <div class="text-lg font-mono font-bold text-orange-500">
            {total > 0n ? (Number((veto * 10000n) / total) / 100).toFixed(2) : "0.00"}%
          </div>
          <div class="text-[10px] text-muted-foreground font-mono">{formatAmount(veto.toString(), tokenExponent)} {tokenSymbol}</div>
        </div>

        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 bg-gray-400 rounded-sm"></span>
            <span class="text-xs font-medium">Abstain</span>
          </div>
          <div class="text-lg font-mono font-bold text-gray-400">
            {total > 0n ? (Number((abstain * 10000n) / total) / 100).toFixed(2) : "0.00"}%
          </div>
          <div class="text-[10px] text-muted-foreground font-mono">{formatAmount(abstain.toString(), tokenExponent)} {tokenSymbol}</div>
        </div>
      </div>
    </div>
  </div>
{/snippet}

{#snippet tallyLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Voting Results")}
    <div class="p-4">
      <Skeleton class="h-32" />
    </div>
  </div>
{/snippet}

{#snippet tallyError(_e: unknown)}
  {@render tallyLoading()}
{/snippet}

{#snippet timelineSection(proposal: Proposal)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Timeline")}

    <div class="p-4">
      <div class="space-y-4">
        <!-- Submit -->
        <div class="flex items-start gap-3">
          <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center shrink-0">
            <CalendarIcon class="h-4 w-4 text-muted-foreground" />
          </div>
          <div>
            <div class="text-xs font-medium">Submitted</div>
            <div class="text-[10px] text-muted-foreground">{formatTime(proposal.submit_time)}</div>
          </div>
        </div>

        <!-- Deposit End -->
        <div class="flex items-start gap-3">
          <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center shrink-0">
            <CoinsIcon class="h-4 w-4 text-muted-foreground" />
          </div>
          <div>
            <div class="text-xs font-medium">Deposit Period End</div>
            <div class="text-[10px] text-muted-foreground">{formatTime(proposal.deposit_end_time)}</div>
            {#if proposal.total_deposit.length > 0}
              <div class="text-[10px] font-mono text-muted-foreground mt-1">
                Deposited: {formatAmount(proposal.total_deposit[0].amount, tokenExponent)} {tokenSymbol}
              </div>
            {/if}
          </div>
        </div>

        <!-- Voting Start -->
        {#if proposal.voting_start_time && proposal.voting_start_time !== "0001-01-01T00:00:00Z"}
          <div class="flex items-start gap-3">
            <div class="w-8 h-8 rounded-full bg-blue-500/20 flex items-center justify-center shrink-0">
              <VoteIcon class="h-4 w-4 text-blue-500" />
            </div>
            <div>
              <div class="text-xs font-medium">Voting Started</div>
              <div class="text-[10px] text-muted-foreground">{formatTime(proposal.voting_start_time)}</div>
            </div>
          </div>
        {/if}

        <!-- Voting End -->
        {#if proposal.voting_end_time && proposal.voting_end_time !== "0001-01-01T00:00:00Z"}
          {@const isVoting = proposal.status.toUpperCase().includes("VOTING")}
          {@const isPast = new Date(proposal.voting_end_time) < new Date()}
          <div class="flex items-start gap-3">
            <div class="w-8 h-8 rounded-full {isPast ? 'bg-muted' : 'bg-blue-500/20'} flex items-center justify-center shrink-0">
              <ClockIcon class="h-4 w-4 {isPast ? 'text-muted-foreground' : 'text-blue-500'}" />
            </div>
            <div>
              <div class="text-xs font-medium">{isPast ? 'Voting Ended' : 'Voting Ends'}</div>
              <div class="text-[10px] text-muted-foreground">{formatTime(proposal.voting_end_time)}</div>
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/snippet}

{#snippet votesSuccess(result: { votes: Array<{ proposal_id: string; voter: string; options: Array<{ option: string; weight: string }> }>; pagination: PaginationResponse })}
  {@const votes = result.votes}

  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center gap-2">
        <UsersIcon class="h-4 w-4 text-muted-foreground" />
        <span class="text-xs font-medium uppercase tracking-wider">Votes</span>
        <Badge variant="default">{votes.length}</Badge>
      </div>
    </div>

    {#if votes.length > 0}
      <!-- Table Header -->
      <div class="grid grid-cols-[1fr_100px] px-4 py-2 border-b border-border bg-muted/10 text-[10px] font-mono uppercase tracking-wider text-muted-foreground">
        <div>Voter</div>
        <div class="text-right">Vote</div>
      </div>

      <div class="divide-y divide-border max-h-96 overflow-y-auto">
        {#each votes as vote}
          {@const mainOption = vote.options[0]}
          {@const voteInfo = mainOption ? getVoteLabel(mainOption.option) : { label: "Unknown", color: "text-muted-foreground" }}
          <div class="grid grid-cols-[1fr_100px] px-4 py-2.5 items-center">
            <a href={urls.account(vote.voter)} class="font-mono text-sm truncate hover:underline">
              {truncateAddress(vote.voter, 12)}
            </a>
            <div class="text-right">
              <span class="text-xs font-mono {voteInfo.color}">{voteInfo.label}</span>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="px-4 py-8 text-center text-muted-foreground text-sm">
        No votes recorded yet
      </div>
    {/if}
  </div>
{/snippet}

{#snippet votesLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Votes")}
    <div class="p-4">
      <Skeleton class="h-32" />
    </div>
  </div>
{/snippet}

{#snippet votesError(_e: unknown)}
  {@render votesLoading()}
{/snippet}

<!-- Pre-load data -->
{@render matchPromiseWithCache(data.stakingPool, {
  cacheKey: `${cachePrefix}staking:pool`,
  onLoading: () => {},
  onSuccess: () => {},
  onError: () => {},
})}
{@render matchPromiseWithCache(data.validators, {
  cacheKey: `${cachePrefix}validators:bonded`,
  onLoading: () => {},
  onSuccess: () => {},
  onError: () => {},
})}

<div class="space-y-6">
  <!-- Proposal Header -->
  {@render matchPromiseWithCache(data.proposal, {
    cacheKey: `${cachePrefix}proposal:${data.id}`,
    onLoading: proposalLoading,
    onSuccess: proposalSuccess,
    onError: proposalError,
  })}

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
    <!-- Tally (2 cols) -->
    <div class="lg:col-span-2">
      {@render matchPromiseWithCache(data.tally, {
        cacheKey: `${cachePrefix}proposal-tally:${data.id}`,
        onLoading: tallyLoading,
        onSuccess: tallySuccess,
        onError: tallyError,
      })}
    </div>

    <!-- Timeline (1 col) -->
    <div>
      {@render matchPromiseWithCache(data.proposal, {
        cacheKey: `${cachePrefix}proposal:${data.id}`,
        onLoading: () => {},
        onSuccess: (result: { proposal: Proposal }) => timelineSection(result.proposal),
        onError: () => {},
      })}
    </div>
  </div>

  <!-- Votes Table -->
  {@render matchPromiseWithCache(data.votes, {
    cacheKey: `${cachePrefix}proposal-votes:${data.id}`,
    onLoading: votesLoading,
    onSuccess: votesSuccess,
    onError: votesError,
  })}
</div>
