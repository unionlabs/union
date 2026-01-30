<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/snippet-cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { Badge, type BadgeVariant } from "$lib/components/ui/badge/index.js"
import { formatAmount, formatTimeAgo } from "$lib/utils/format"
import { urls } from "$lib/utils/urls"
import type { Proposal, StakingPool, PaginationResponse } from "$lib/types/cosmos"
import VoteIcon from "@lucide/svelte/icons/vote"
import CheckCircleIcon from "@lucide/svelte/icons/check-circle"
import XCircleIcon from "@lucide/svelte/icons/x-circle"
import ClockIcon from "@lucide/svelte/icons/clock"
import AlertCircleIcon from "@lucide/svelte/icons/alert-circle"
import BanIcon from "@lucide/svelte/icons/ban"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

let activeTab = $state<"all" | "voting" | "passed" | "rejected">("all")

onMount(() => {
  const interval = setInterval(() => {
    invalidate("governance:data")
  }, 30_000)
  return () => clearInterval(interval)
})

// Status mapping
const getStatusInfo = (status: string): { label: string; variant: BadgeVariant; icon: typeof VoteIcon } => {
  const s = status.toUpperCase()
  if (s.includes("VOTING")) return { label: "Voting", variant: "info", icon: VoteIcon }
  if (s.includes("PASSED")) return { label: "Passed", variant: "success", icon: CheckCircleIcon }
  if (s.includes("REJECTED")) return { label: "Rejected", variant: "destructive", icon: XCircleIcon }
  if (s.includes("DEPOSIT")) return { label: "Deposit", variant: "warning", icon: ClockIcon }
  if (s.includes("FAILED")) return { label: "Failed", variant: "destructive", icon: AlertCircleIcon }
  return { label: status.replace("PROPOSAL_STATUS_", ""), variant: "secondary", icon: AlertCircleIcon }
}

// Get proposal type from messages
const getProposalType = (proposal: Proposal): string => {
  if (proposal.messages.length === 0) return "Text"
  const type = proposal.messages[0]["@type"] ?? ""
  if (type.includes("SoftwareUpgrade")) return "Upgrade"
  if (type.includes("ParameterChange") || type.includes("MsgUpdateParams")) return "Params"
  if (type.includes("CommunityPool")) return "Spend"
  if (type.includes("CancelUpgrade")) return "Cancel"
  if (type.includes("ClientUpdate")) return "IBC Client"
  return type.split(".").pop()?.replace("Msg", "") ?? "Text"
}

// Calculate tally percentages
const getTallyPercentages = (proposal: Proposal) => {
  const tally = proposal.final_tally_result
  const yes = BigInt(tally.yes_count || "0")
  const no = BigInt(tally.no_count || "0")
  const abstain = BigInt(tally.abstain_count || "0")
  const veto = BigInt(tally.no_with_veto_count || "0")
  const total = yes + no + abstain + veto

  if (total === 0n) {
    return { yes: 0, no: 0, abstain: 0, veto: 0, total: 0n }
  }

  return {
    yes: Number((yes * 10000n) / total) / 100,
    no: Number((no * 10000n) / total) / 100,
    abstain: Number((abstain * 10000n) / total) / 100,
    veto: Number((veto * 10000n) / total) / 100,
    total,
  }
}

// Filter proposals by tab
const filterProposals = (proposals: Proposal[], tab: string) => {
  if (tab === "all") return proposals
  return proposals.filter(p => {
    const s = p.status.toUpperCase()
    if (tab === "voting") return s.includes("VOTING")
    if (tab === "passed") return s.includes("PASSED")
    if (tab === "rejected") return s.includes("REJECTED") || s.includes("FAILED")
    return true
  })
}

// Time remaining for voting
const getTimeRemaining = (endTime: string): string => {
  const end = new Date(endTime)
  const now = new Date()
  const diff = end.getTime() - now.getTime()

  if (diff <= 0) return "Ended"

  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
  const mins = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))

  if (days > 0) return `${days}d ${hours}h`
  if (hours > 0) return `${hours}h ${mins}m`
  return `${mins}m`
}

const getCachedPool = () => cache.get(`${cachePrefix}staking:pool`) as { pool: StakingPool } | undefined
const getCachedProposals = () => cache.get(`${cachePrefix}proposals:all`) as { proposals: Proposal[] } | undefined
</script>


{#snippet tallyBar(proposal: Proposal)}
  {@const tally = getTallyPercentages(proposal)}
  <div class="flex h-1.5 w-full overflow-hidden bg-muted/50 rounded-full">
    {#if tally.total > 0n}
      <div class="bg-green-500 h-full" style="width: {tally.yes}%"></div>
      <div class="bg-red-500 h-full" style="width: {tally.no}%"></div>
      <div class="bg-orange-500 h-full" style="width: {tally.veto}%"></div>
      <div class="bg-gray-400 h-full" style="width: {tally.abstain}%"></div>
    {/if}
  </div>
{/snippet}

{#snippet proposalsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Proposals</span>
    </div>
    <div class="p-4 space-y-3">
      {#each Array(5) as _}
        <Skeleton class="h-24" />
      {/each}
    </div>
  </div>
{/snippet}

{#snippet proposalsSuccess(result: { proposals: Proposal[]; pagination: PaginationResponse })}
  {@const allProposals = result.proposals.sort((a, b) => Number(b.id) - Number(a.id))}
  {@const proposals = filterProposals(allProposals, activeTab)}
  {@const votingCount = allProposals.filter(p => p.status.toUpperCase().includes("VOTING")).length}
  {@const passedCount = allProposals.filter(p => p.status.toUpperCase().includes("PASSED")).length}
  {@const rejectedCount = allProposals.filter(p => p.status.toUpperCase().includes("REJECTED") || p.status.toUpperCase().includes("FAILED")).length}

  <div class="relative border border-border">
    <CornerMarks />

    <!-- Header with tabs -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center gap-2">
        <VoteIcon class="h-4 w-4 text-muted-foreground" />
        <span class="text-xs font-medium uppercase tracking-wider">Governance</span>
        <Badge variant="default">{allProposals.length}</Badge>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-border bg-muted/10">
      <button
        onclick={() => activeTab = "all"}
        class="px-4 py-2 text-xs font-medium transition-colors {activeTab === 'all' ? 'text-foreground border-b-2 border-foreground -mb-px' : 'text-muted-foreground hover:text-foreground'}"
      >
        All ({allProposals.length})
      </button>
      <button
        onclick={() => activeTab = "voting"}
        class="px-4 py-2 text-xs font-medium transition-colors {activeTab === 'voting' ? 'text-foreground border-b-2 border-foreground -mb-px' : 'text-muted-foreground hover:text-foreground'}"
      >
        Voting ({votingCount})
      </button>
      <button
        onclick={() => activeTab = "passed"}
        class="px-4 py-2 text-xs font-medium transition-colors {activeTab === 'passed' ? 'text-foreground border-b-2 border-foreground -mb-px' : 'text-muted-foreground hover:text-foreground'}"
      >
        Passed ({passedCount})
      </button>
      <button
        onclick={() => activeTab = "rejected"}
        class="px-4 py-2 text-xs font-medium transition-colors {activeTab === 'rejected' ? 'text-foreground border-b-2 border-foreground -mb-px' : 'text-muted-foreground hover:text-foreground'}"
      >
        Rejected ({rejectedCount})
      </button>
    </div>

    <!-- Proposals List -->
    {#if proposals.length > 0}
      <div class="divide-y divide-border">
        {#each proposals as proposal}
          {@const status = getStatusInfo(proposal.status)}
          {@const type = getProposalType(proposal)}
          {@const tally = getTallyPercentages(proposal)}
          {@const isVoting = proposal.status.toUpperCase().includes("VOTING")}
          <a
            href={urls.proposal(proposal.id)}
            class="block px-4 py-4 hover:bg-muted/30 transition-colors"
          >
            <div class="flex items-start justify-between gap-4 mb-3">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-[10px] font-mono text-muted-foreground">#{proposal.id}</span>
                  <Badge variant="default">{type}</Badge>
                  <Badge variant={status.variant}>{status.label}</Badge>
                </div>
                <h3 class="text-sm font-medium truncate">{proposal.title || "Untitled Proposal"}</h3>
                {#if proposal.summary}
                  <p class="text-xs text-muted-foreground mt-1 line-clamp-2">{proposal.summary}</p>
                {/if}
              </div>
              {#if isVoting}
                <div class="text-right shrink-0">
                  <div class="text-[10px] font-mono text-muted-foreground uppercase">Ends in</div>
                  <div class="text-sm font-mono font-medium text-blue-500">{getTimeRemaining(proposal.voting_end_time)}</div>
                </div>
              {/if}
            </div>

            <!-- Tally bar -->
            <div class="space-y-1.5">
              {@render tallyBar(proposal)}
              <div class="flex items-center justify-between text-[10px] font-mono text-muted-foreground">
                <div class="flex items-center gap-3">
                  <span class="flex items-center gap-1">
                    <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                    Yes {tally.yes.toFixed(1)}%
                  </span>
                  <span class="flex items-center gap-1">
                    <span class="w-2 h-2 bg-red-500 rounded-full"></span>
                    No {tally.no.toFixed(1)}%
                  </span>
                  <span class="flex items-center gap-1">
                    <span class="w-2 h-2 bg-orange-500 rounded-full"></span>
                    Veto {tally.veto.toFixed(1)}%
                  </span>
                  <span class="flex items-center gap-1">
                    <span class="w-2 h-2 bg-gray-400 rounded-full"></span>
                    Abstain {tally.abstain.toFixed(1)}%
                  </span>
                </div>
                <span>{formatTimeAgo(proposal.submit_time)}</span>
              </div>
            </div>
          </a>
        {/each}
      </div>
    {:else}
      <div class="px-4 py-12 text-center text-muted-foreground text-sm">
        No proposals found
      </div>
    {/if}
  </div>
{/snippet}

{#snippet proposalsError(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-xs font-medium uppercase tracking-wider">Proposals</span>
    </div>
    <div class="px-4 py-12 text-center text-muted-foreground text-sm">
      Failed to load proposals
    </div>
  </div>
{/snippet}

{#snippet statsSection()}
  {@const pool = getCachedPool()}
  {@const cachedProposals = getCachedProposals()}
  {@const proposals = cachedProposals?.proposals ?? []}
  {@const voting = proposals.filter(p => p.status.toUpperCase().includes("VOTING"))}
  {@const passed = proposals.filter(p => p.status.toUpperCase().includes("PASSED"))}

  <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
    <!-- Total Proposals -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Total Proposals</div>
      {#if cachedProposals}
        <div class="text-xl font-mono font-bold">{proposals.length}</div>
        <div class="text-xs text-muted-foreground">all time</div>
      {:else}
        <Skeleton class="h-7 w-16" />
      {/if}
    </div>

    <!-- Active Voting -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Active Voting</div>
      {#if cachedProposals}
        <div class="text-xl font-mono font-bold {voting.length > 0 ? 'text-blue-500' : ''}">{voting.length}</div>
        <div class="text-xs text-muted-foreground">proposals</div>
      {:else}
        <Skeleton class="h-7 w-16" />
      {/if}
    </div>

    <!-- Passed -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Passed</div>
      {#if cachedProposals}
        <div class="text-xl font-mono font-bold text-green-500">{passed.length}</div>
        <div class="text-xs text-muted-foreground">proposals</div>
      {:else}
        <Skeleton class="h-7 w-16" />
      {/if}
    </div>

    <!-- Pass Rate -->
    <div class="relative border border-border p-4">
      <CornerMarks />
      <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-2">Pass Rate</div>
      {#if cachedProposals}
        {@const completed = proposals.filter(p => !p.status.toUpperCase().includes("VOTING") && !p.status.toUpperCase().includes("DEPOSIT"))}
        {@const rate = completed.length > 0 ? (passed.length / completed.length * 100) : 0}
        <div class="text-xl font-mono font-bold">{rate.toFixed(0)}%</div>
        <div class="text-xs text-muted-foreground">of completed</div>
      {:else}
        <Skeleton class="h-7 w-16" />
      {/if}
    </div>
  </div>
{/snippet}

<!-- Pre-load staking pool for stats -->
{@render matchPromiseWithCache(data.stakingPool, {
  cacheKey: `${cachePrefix}staking:pool`,
  onLoading: () => {},
  onSuccess: () => {},
  onError: () => {},
})}

<div class="space-y-6">
  <!-- Stats -->
  {@render statsSection()}

  <!-- Proposals List -->
  {@render matchPromiseWithCache(data.proposals, {
    cacheKey: `${cachePrefix}proposals:all`,
    onLoading: proposalsLoading,
    onSuccess: proposalsSuccess,
    onError: proposalsError,
  })}
</div>
