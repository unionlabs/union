<script lang="ts">
import { onMount } from "svelte"
import { invalidate } from "$app/navigation"
import { matchPromiseWithCache, cache } from "$lib/snippet-cache/promise.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { sectionHeader, statCard, statCardLoading } from "$lib/components/ui/snippets.svelte"
import { Badge } from "$lib/components/ui/badge/index.js"
import * as Collapsible from "$lib/components/ui/collapsible/index.js"
import { formatAmount, formatTimeAgo, formatTime, truncateAddress, truncateHash } from "$lib/utils/format"
import { getMsgType, getMsgTypeVariant } from "$lib/utils/messages"
import { copyToClipboard as copyUtil } from "$lib/utils/clipboard"
import { urls } from "$lib/utils/urls"
import type { Account, Coin, Delegation, UnbondingDelegation, TxResponse, Validator, PaginationResponse } from "$lib/types/cosmos"
import { runPromise } from "$lib/runtime"
import { fetchAllAccountTxs } from "$lib/queries/accounts"
import CopyIcon from "@lucide/svelte/icons/copy"
import CheckIcon from "@lucide/svelte/icons/check"
import XIcon from "@lucide/svelte/icons/x"
import ChevronDownIcon from "@lucide/svelte/icons/chevron-down"
import WalletIcon from "@lucide/svelte/icons/wallet"
import CoinsIcon from "@lucide/svelte/icons/coins"
import LockIcon from "@lucide/svelte/icons/lock"
import GiftIcon from "@lucide/svelte/icons/gift"
import ClockIcon from "@lucide/svelte/icons/clock"
import ArrowUpIcon from "@lucide/svelte/icons/arrow-up"
import ArrowDownIcon from "@lucide/svelte/icons/arrow-down"
import ArrowRightLeftIcon from "@lucide/svelte/icons/arrow-right-left"
import LoaderIcon from "@lucide/svelte/icons/loader"
import CornerMarks from "$lib/components/corner-marks.svelte"

const { data } = $props()

// Chain-specific cache key prefix
const cachePrefix = $derived(`${data.chain.universal_chain_id}:`)

// Get token info from chain config
const tokenSymbol = $derived(data.chain.assets[0]?.symbol ?? "TOKEN")
const tokenBase = $derived(data.chain.assets[0]?.base ?? "")
const tokenExponent = $derived(data.chain.assets[0]?.exponent ?? 6)

let copied = $state(false)

// Pagination state for unified transactions
let extraTxs = $state<TxResponse[]>([])
let currentPage = $state(1)
let loadingMore = $state(false)
let hasMore = $state(true)
const PAGE_SIZE = 25

async function loadMoreTxs() {
  loadingMore = true
  try {
    const nextPage = currentPage + 1
    const result = await runPromise(fetchAllAccountTxs(data.address, nextPage, PAGE_SIZE))
    if (result.tx_responses.length > 0) {
      // Filter out duplicates based on txhash
      const existingHashes = new Set(extraTxs.map(tx => tx.txhash))
      const newTxs = result.tx_responses.filter(tx => !existingHashes.has(tx.txhash))
      extraTxs = [...extraTxs, ...newTxs]
      currentPage = nextPage
    }
    hasMore = result.hasMore
  } catch (e) {
    console.error("Failed to load more txs:", e)
  } finally {
    loadingMore = false
  }
}

onMount(() => {
  const interval = setInterval(() => {
    invalidate("account:data")
  }, 30_000)
  return () => clearInterval(interval)
})

function copyToClipboard(text: string) {
  copyUtil(text, () => {
    copied = true
    setTimeout(() => copied = false, 2000)
  })
}

// Get account type from @type field
const getAccountType = (type: string): string => {
  if (type.includes("BaseAccount")) return "Base Account"
  if (type.includes("ModuleAccount")) return "Module Account"
  if (type.includes("BaseVestingAccount")) return "Vesting Account"
  if (type.includes("ContinuousVestingAccount")) return "Continuous Vesting"
  if (type.includes("DelayedVestingAccount")) return "Delayed Vesting"
  if (type.includes("PeriodicVestingAccount")) return "Periodic Vesting"
  if (type.includes("PermanentLockedAccount")) return "Permanent Locked"
  return type.split(".").pop() ?? "Unknown"
}

// Get validator name from cache
const getValidatorName = (validatorAddress: string): string => {
  const cached = cache.get("validators:bonded") as { validators: Validator[] } | undefined
  if (cached?.validators) {
    const validator = cached.validators.find((v) => v.operator_address === validatorAddress)
    if (validator) return validator.description.moniker
  }
  return truncateAddress(validatorAddress, 8)
}

// Calculate time remaining for unbonding
const getTimeRemaining = (completionTime: string): string => {
  const completion = new Date(completionTime)
  const now = new Date()
  const diff = completion.getTime() - now.getTime()

  if (diff <= 0) return "Complete"

  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))

  if (days > 0) return `${days}d ${hours}h`
  return `${hours}h`
}

// Parse amount string like "1000000utoken" -> formatted
const parseAmountString = (amountStr: string): string => {
  const match = amountStr.match(/^(\d+)(.*)$/)
  if (match) {
    const [, amount, denom] = match
    if (denom === tokenBase) {
      return `${formatAmount(amount, tokenExponent)} ${tokenSymbol}`
    }
    return `${formatAmount(amount, 6)} ${denom}`
  }
  return amountStr
}

// Extract sent/spent amount from transaction message
const getSentAmount = (tx: TxResponse): string => {
  const msg = tx.tx.body.messages[0] as Record<string, unknown>
  const type = getMsgType(msg as { "@type": string }).toLowerCase()

  // Helper to format coin
  const formatCoin = (denom: string, amount: string): string => {
    if (denom === tokenBase) return `${formatAmount(amount, tokenExponent)} ${tokenSymbol}`
    return `${formatAmount(amount, 6)} ${denom}`
  }

  // MsgSend - direct transfer
  if (type === "send" && msg.amount) {
    const amounts = msg.amount as Array<{ denom: string; amount: string }>
    if (amounts.length > 0) {
      const { denom, amount } = amounts[0]
      return formatCoin(denom, amount)
    }
  }

  // MsgDelegate, MsgUndelegate, MsgBeginRedelegate
  if ((type.includes("delegate") || type.includes("redelegate")) && msg.amount) {
    const amt = msg.amount as { denom: string; amount: string }
    return formatCoin(amt.denom, amt.amount)
  }

  // IBC Transfer
  if (type === "transfer" && msg.token) {
    const token = msg.token as { denom: string; amount: string }
    return formatCoin(token.denom, token.amount)
  }

  // Try to get from coin_spent event
  const coinSpentEvent = tx.events?.find(e => e.type === "coin_spent")
  if (coinSpentEvent) {
    const amountAttr = coinSpentEvent.attributes.find(a => a.key === "amount")
    if (amountAttr?.value) {
      return parseAmountString(amountAttr.value)
    }
  }

  return "-"
}

// Extract received amount from transaction events for a specific address
const getReceivedAmount = (tx: TxResponse, address?: string): string => {
  // Find coin_received event for this specific address if provided
  const coinReceivedEvent = address
    ? tx.events?.find(e =>
        e.type === "coin_received" &&
        e.attributes.some(a => a.key === "receiver" && a.value === address)
      )
    : tx.events?.find(e => e.type === "coin_received")

  if (coinReceivedEvent) {
    const amountAttr = coinReceivedEvent.attributes.find(a => a.key === "amount")
    if (amountAttr?.value) {
      return parseAmountString(amountAttr.value)
    }
  }
  return "-"
}

// Determine transaction direction relative to the current address
type TxDirection = "sent" | "received" | "self"

const getTxDirection = (tx: TxResponse, address: string): TxDirection => {
  const msg = tx.tx.body.messages[0] as Record<string, unknown>

  // Check if sender
  const sender = msg.sender || msg.from_address || msg.delegator_address
  const isSender = sender === address

  // Check if receiver
  const receiver = msg.to_address || msg.receiver || msg.validator_address
  const isReceiver = receiver === address

  // Check events for more accurate direction
  const coinReceived = tx.events?.find(e =>
    e.type === "coin_received" &&
    e.attributes.some(a => a.key === "receiver" && a.value === address)
  )
  const coinSpent = tx.events?.find(e =>
    e.type === "coin_spent" &&
    e.attributes.some(a => a.key === "spender" && a.value === address)
  )

  if (isSender && isReceiver) return "self"
  if (coinReceived && !coinSpent) return "received"
  if (coinSpent && !coinReceived) return "sent"
  if (isSender) return "sent"
  if (isReceiver || coinReceived) return "received"

  return "sent" // Default to sent if unclear
}

// Get amount with direction context
const getTxAmount = (tx: TxResponse, direction: TxDirection, address: string): string => {
  if (direction === "received") {
    return getReceivedAmount(tx, address)
  }
  return getSentAmount(tx)
}
</script>


{#snippet accountLoading()}
  <div class="space-y-6">
    <div class="relative border border-border">
      <CornerMarks />
      <div class="p-6">
        <Skeleton class="h-24 w-full" />
      </div>
    </div>
    <div class="grid grid-cols-4 gap-4">
      {#each Array(4) as _}
        <div class="relative border border-border">
          <CornerMarks />
          <div class="p-4">
            <Skeleton class="h-16" />
          </div>
        </div>
      {/each}
    </div>
  </div>
{/snippet}

{#snippet accountSuccess(result: { account: Account })}
  {@const account = result.account}
  {@const accountType = getAccountType(account["@type"])}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-12 h-12 bg-muted flex items-center justify-center">
              <WalletIcon class="h-6 w-6 text-muted-foreground" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <Badge variant="default">{accountType}</Badge>
              </div>
              <div class="flex items-center gap-2">
                <span class="font-mono text-sm break-all">{account.address}</span>
                <button
                  onclick={() => copyToClipboard(account.address)}
                  class="p-1 hover:bg-muted transition-colors shrink-0"
                >
                  {#if copied}
                    <CheckIcon class="h-4 w-4 text-success" />
                  {:else}
                    <CopyIcon class="h-4 w-4 text-muted-foreground" />
                  {/if}
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="text-right shrink-0 ml-4">
          <div class="grid grid-cols-2 gap-x-6 gap-y-2">
            <div>
              <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground">Account #</div>
              <div class="font-mono text-sm">{account.account_number}</div>
            </div>
            <div>
              <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground">Sequence</div>
              <div class="font-mono text-sm">{account.sequence}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/snippet}

{#snippet accountError(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="p-6">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="flex items-center gap-3 mb-3">
            <div class="w-12 h-12 bg-muted flex items-center justify-center">
              <WalletIcon class="h-6 w-6 text-muted-foreground" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <Badge variant="default">New Account</Badge>
              </div>
              <div class="flex items-center gap-2">
                <span class="font-mono text-sm break-all">{data.address}</span>
                <button
                  onclick={() => copyToClipboard(data.address)}
                  class="p-1 hover:bg-muted transition-colors shrink-0"
                >
                  {#if copied}
                    <CheckIcon class="h-4 w-4 text-success" />
                  {:else}
                    <CopyIcon class="h-4 w-4 text-muted-foreground" />
                  {/if}
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="text-right shrink-0 ml-4">
          <span class="text-xs text-muted-foreground">Account not found on chain</span>
        </div>
      </div>
    </div>
  </div>
{/snippet}

{#snippet balancesLoading()}
  <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
    {#each Array(4) as _}
      <div class="relative border border-border">
        <CornerMarks />
        <div class="p-4">
          <Skeleton class="h-16" />
        </div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet balancesSuccess(result: { balances: Coin[]; pagination: PaginationResponse })}
  {@const nativeBalance = result.balances.find(b => b.denom === tokenBase)}
  {@const available = nativeBalance ? formatAmount(nativeBalance.amount, tokenExponent) : "0.00"}
  {@const otherBalances = result.balances.filter(b => b.denom !== tokenBase)}

  <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
    {@render statCard(CoinsIcon, "Available", `${available} ${tokenSymbol}`)}
    {@render matchPromiseWithCache(data.delegations, {
      cacheKey: `${cachePrefix}account:${data.address}:delegations`,
      onLoading: () => statCard(LockIcon, "Delegated", "...", "Loading"),
      onSuccess: delegatedBalanceSuccess,
      onError: () => statCard(LockIcon, "Delegated", `0.00 ${tokenSymbol}`),
    })}
    {@render matchPromiseWithCache(data.rewards, {
      cacheKey: `${cachePrefix}account:${data.address}:rewards`,
      onLoading: () => statCard(GiftIcon, "Rewards", "...", "Loading"),
      onSuccess: rewardsBalanceSuccess,
      onError: () => statCard(GiftIcon, "Rewards", `0.00 ${tokenSymbol}`),
    })}
    {@render matchPromiseWithCache(data.unbonding, {
      cacheKey: `${cachePrefix}account:${data.address}:unbonding`,
      onLoading: () => statCard(ClockIcon, "Unbonding", "...", "Loading"),
      onSuccess: unbondingBalanceSuccess,
      onError: () => statCard(ClockIcon, "Unbonding", `0.00 ${tokenSymbol}`),
    })}
  </div>

  {#if otherBalances.length > 0}
    <div class="relative border border-border mt-4">
      <CornerMarks />
      {@render sectionHeader("Other Assets")}
      <div class="divide-y divide-border">
        {#each otherBalances as coin}
          <div class="grid grid-cols-[1fr_200px] px-4 py-2.5">
            <span class="font-mono text-sm truncate">{coin.denom.startsWith("ibc/") ? truncateHash(coin.denom, 8) : coin.denom}</span>
            <span class="font-mono text-sm text-right">{formatAmount(coin.amount, 6)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/snippet}

{#snippet balancesError(_e: unknown)}
  <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
    {@render statCard(CoinsIcon, "Available", `0.00 ${tokenSymbol}`)}
    {@render statCard(LockIcon, "Delegated", `0.00 ${tokenSymbol}`)}
    {@render statCard(GiftIcon, "Rewards", `0.00 ${tokenSymbol}`)}
    {@render statCard(ClockIcon, "Unbonding", `0.00 ${tokenSymbol}`)}
  </div>
{/snippet}

{#snippet delegatedBalanceSuccess(result: { delegation_responses: Delegation[]; pagination: PaginationResponse })}
  {@const total = result.delegation_responses.reduce((sum, d) => sum + BigInt(d.balance.amount), 0n)}
  {@render statCard(LockIcon, "Delegated", `${formatAmount(total.toString(), tokenExponent)} ${tokenSymbol}`, `${result.delegation_responses.length} validators`)}
{/snippet}

{#snippet rewardsBalanceSuccess(result: { rewards: Array<{ validator_address: string; reward: Coin[] }>; total: Coin[] })}
  {@const nativeReward = result.total.find(c => c.denom === tokenBase)}
  {@render statCard(GiftIcon, "Rewards", `${nativeReward ? formatAmount(nativeReward.amount, tokenExponent) : "0.00"} ${tokenSymbol}`, "Claimable")}
{/snippet}

{#snippet unbondingBalanceSuccess(result: { unbonding_responses: UnbondingDelegation[]; pagination: PaginationResponse })}
  {@const total = result.unbonding_responses.reduce((sum, u) => sum + u.entries.reduce((s, e) => s + BigInt(e.balance), 0n), 0n)}
  {@const entryCount = result.unbonding_responses.reduce((sum, u) => sum + u.entries.length, 0)}
  {@render statCard(ClockIcon, "Unbonding", `${formatAmount(total.toString(), tokenExponent)} ${tokenSymbol}`, entryCount > 0 ? `${entryCount} pending` : undefined)}
{/snippet}

{#snippet delegationsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Delegations", "01")}
    <div class="p-4">
      <Skeleton class="h-32" />
    </div>
  </div>
{/snippet}

{#snippet delegationsSuccess(result: { delegation_responses: Delegation[]; pagination: PaginationResponse })}
  {@const delegations = result.delegation_responses}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader(`Delegations (${delegations.length})`, "01")}
    {#if delegations.length > 0}
      <div class="divide-y divide-border">
        {#each delegations as delegation}
          {@const validatorName = getValidatorName(delegation.delegation.validator_address)}
          <a
            href={urls.validator(delegation.delegation.validator_address)}
            class="grid grid-cols-[1fr_150px] px-4 py-3 hover:bg-muted/30 transition-colors"
          >
            <div>
              <div class="font-medium text-sm">{validatorName}</div>
              <div class="font-mono text-xs text-muted-foreground">{truncateAddress(delegation.delegation.validator_address, 10)}</div>
            </div>
            <div class="text-right">
              <div class="font-mono text-sm font-medium">{formatAmount(delegation.balance.amount, tokenExponent)} {tokenSymbol}</div>
            </div>
          </a>
        {/each}
      </div>
    {:else}
      <div class="px-4 py-8 text-center text-muted-foreground text-sm">
        No delegations
      </div>
    {/if}
  </div>
{/snippet}

{#snippet delegationsError(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Delegations", "01")}
    <div class="px-4 py-8 text-center text-muted-foreground text-sm">
      No delegations
    </div>
  </div>
{/snippet}

{#snippet unbondingLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Unbonding", "02")}
    <div class="p-4">
      <Skeleton class="h-20" />
    </div>
  </div>
{/snippet}

{#snippet unbondingSuccess(result: { unbonding_responses: UnbondingDelegation[]; pagination: PaginationResponse })}
  {@const unbondings = result.unbonding_responses}
  {@const hasEntries = unbondings.some(u => u.entries.length > 0)}
  {#if hasEntries}
    <div class="relative border border-border">
      <CornerMarks />
      {@render sectionHeader("Unbonding", "02")}
      <div class="divide-y divide-border">
        {#each unbondings as unbonding}
          {#each unbonding.entries as entry}
            {@const validatorName = getValidatorName(unbonding.validator_address)}
            {@const timeRemaining = getTimeRemaining(entry.completion_time)}
            <div class="grid grid-cols-[1fr_120px_100px] px-4 py-3">
              <div>
                <div class="font-medium text-sm">{validatorName}</div>
                <div class="font-mono text-xs text-muted-foreground">{truncateAddress(unbonding.validator_address, 10)}</div>
              </div>
              <div class="text-right font-mono text-sm">{formatAmount(entry.balance, tokenExponent)} {tokenSymbol}</div>
              <div class="text-right">
                <Badge variant="warning">{timeRemaining}</Badge>
              </div>
            </div>
          {/each}
        {/each}
      </div>
    </div>
  {/if}
{/snippet}

{#snippet unbondingError(_e: unknown)}
  <!-- No unbonding section on error -->
{/snippet}

{#snippet rewardsLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Pending Rewards", "03")}
    <div class="p-4">
      <Skeleton class="h-20" />
    </div>
  </div>
{/snippet}

{#snippet rewardsSuccess(result: { rewards: Array<{ validator_address: string; reward: Coin[] }>; total: Coin[] })}
  {@const rewards = result.rewards.filter(r => r.reward.length > 0)}
  {#if rewards.length > 0}
    <Collapsible.Root>
      <div class="relative border border-border">
        <CornerMarks />
        <Collapsible.Trigger class="w-full">
          <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20 hover:bg-muted/40 transition-colors">
            <div class="flex items-center gap-3">
              <span class="text-[10px] font-mono text-muted-foreground">03</span>
              <span class="text-xs font-medium uppercase tracking-wider">Pending Rewards</span>
              <Badge variant="default">{rewards.length} validators</Badge>
            </div>
            <ChevronDownIcon class="h-4 w-4 text-muted-foreground" />
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="divide-y divide-border">
            {#each rewards as reward}
              {@const validatorName = getValidatorName(reward.validator_address)}
              {@const nativeReward = reward.reward.find(c => c.denom === tokenBase)}
              <div class="grid grid-cols-[1fr_150px] px-4 py-2.5">
                <div>
                  <div class="font-medium text-sm">{validatorName}</div>
                </div>
                <div class="text-right font-mono text-sm text-success">
                  +{nativeReward ? formatAmount(nativeReward.amount, tokenExponent) : "0.00"} {tokenSymbol}
                </div>
              </div>
            {/each}
          </div>
        </Collapsible.Content>
      </div>
    </Collapsible.Root>
  {/if}
{/snippet}

{#snippet rewardsError(_e: unknown)}
  <!-- No rewards section on error -->
{/snippet}

{#snippet timelineLoading()}
  <div class="relative border border-border">
    <CornerMarks />
    {@render sectionHeader("Transaction History", "04")}
    <div class="p-4">
      <Skeleton class="h-48" />
    </div>
  </div>
{/snippet}

{#snippet timelineSuccess(result: { tx_responses: TxResponse[]; hasMore: boolean })}
  {@const initialTxs = result.tx_responses ?? []}
  {@const allTxs = [...initialTxs, ...extraTxs]}
  <!-- Deduplicate and sort -->
  {@const txMap = new Map(allTxs.map(tx => [tx.txhash, tx]))}
  {@const txs = Array.from(txMap.values()).sort((a, b) => parseInt(b.height) - parseInt(a.height))}
  {@const showLoadMore = hasMore || result.hasMore}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-[10px] font-mono text-muted-foreground">04</span>
      <ArrowRightLeftIcon class="h-3.5 w-3.5 text-muted-foreground" />
      <span class="text-xs font-medium uppercase tracking-wider">Transaction History ({txs.length}{showLoadMore ? '+' : ''})</span>
    </div>
    {#if txs.length > 0}
      <!-- Table Header -->
      <div class="grid grid-cols-[28px_70px_1fr_90px_110px_90px] px-4 py-2 text-[10px] font-mono uppercase tracking-wider text-muted-foreground border-b border-border bg-muted/10">
        <span></span>
        <span>Height</span>
        <span>Hash</span>
        <span>Type</span>
        <span class="text-right">Amount</span>
        <span class="text-right">Time</span>
      </div>
      <div class="divide-y divide-border">
        {#each txs as tx}
          {@const msg = tx.tx.body.messages[0] as { "@type": string; [key: string]: unknown }}
          {@const type = getMsgType(msg)}
          {@const direction = getTxDirection(tx, data.address)}
          {@const amount = getTxAmount(tx, direction, data.address)}
          <div class="grid grid-cols-[28px_70px_1fr_90px_110px_90px] px-4 py-2.5 hover:bg-muted/30 transition-colors items-center">
            <!-- Direction indicator -->
            <div class="flex items-center justify-center">
              {#if direction === "received"}
                <ArrowDownIcon class="h-3.5 w-3.5 text-success" />
              {:else if direction === "sent"}
                <ArrowUpIcon class="h-3.5 w-3.5 text-destructive" />
              {:else}
                <ArrowRightLeftIcon class="h-3 w-3 text-muted-foreground" />
              {/if}
            </div>
            <a href={urls.block(tx.height)} class="font-mono text-sm hover:underline">{tx.height}</a>
            <div class="flex items-center gap-2 min-w-0">
              <a href={urls.transaction(tx.txhash)} class="font-mono text-sm truncate hover:underline">{truncateHash(tx.txhash, 8)}</a>
              {#if tx.code === 0}
                <CheckIcon class="h-3 w-3 text-success shrink-0" />
              {:else}
                <XIcon class="h-3 w-3 text-destructive shrink-0" />
              {/if}
            </div>
            <Badge variant={getMsgTypeVariant(type)}>{type}</Badge>
            <span class="font-mono text-sm text-right {direction === 'received' ? 'text-success' : direction === 'sent' ? 'text-destructive' : 'text-muted-foreground'}">
              {#if amount !== '-'}
                {direction === 'received' ? '+' : direction === 'sent' ? '-' : ''}{amount}
              {:else}
                -
              {/if}
            </span>
            <span class="text-xs text-muted-foreground text-right">{formatTimeAgo(tx.timestamp)}</span>
          </div>
        {/each}
      </div>
      {#if showLoadMore}
        <button
          onclick={loadMoreTxs}
          disabled={loadingMore}
          class="w-full px-4 py-3 text-sm font-medium text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors border-t border-border flex items-center justify-center gap-2 disabled:opacity-50"
        >
          {#if loadingMore}
            <LoaderIcon class="h-4 w-4 animate-spin" />
            Loading...
          {:else}
            Load More
          {/if}
        </button>
      {/if}
    {:else}
      <div class="px-4 py-8 text-center text-muted-foreground text-sm">
        No transactions found
      </div>
    {/if}
  </div>
{/snippet}

{#snippet timelineError(_e: unknown)}
  <div class="relative border border-border">
    <CornerMarks />
    <div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-muted/20">
      <span class="text-[10px] font-mono text-muted-foreground">04</span>
      <ArrowRightLeftIcon class="h-3.5 w-3.5 text-muted-foreground" />
      <span class="text-xs font-medium uppercase tracking-wider">Transaction History</span>
    </div>
    <div class="px-4 py-8 text-center text-muted-foreground text-sm">
      Failed to load transactions
    </div>
  </div>
{/snippet}

<!-- Pre-load validators for name lookup -->
{@render matchPromiseWithCache(data.validators, {
  cacheKey: `${cachePrefix}validators:bonded`,
  onLoading: () => {},
  onSuccess: () => {},
  onError: () => {},
})}

<div class="space-y-6">
  <!-- Account Header -->
  {@render matchPromiseWithCache(data.account, {
    cacheKey: `${cachePrefix}account:${data.address}:info`,
    onLoading: accountLoading,
    onSuccess: accountSuccess,
    onError: accountError,
  })}

  <!-- Portfolio Balances -->
  {@render matchPromiseWithCache(data.balances, {
    cacheKey: `${cachePrefix}account:${data.address}:balances`,
    onLoading: balancesLoading,
    onSuccess: balancesSuccess,
    onError: balancesError,
  })}

  <!-- Delegations -->
  {@render matchPromiseWithCache(data.delegations, {
    cacheKey: `${cachePrefix}account:${data.address}:delegations`,
    onLoading: delegationsLoading,
    onSuccess: delegationsSuccess,
    onError: delegationsError,
  })}

  <!-- Unbonding -->
  {@render matchPromiseWithCache(data.unbonding, {
    cacheKey: `${cachePrefix}account:${data.address}:unbonding`,
    onLoading: unbondingLoading,
    onSuccess: unbondingSuccess,
    onError: unbondingError,
  })}

  <!-- Rewards -->
  {@render matchPromiseWithCache(data.rewards, {
    cacheKey: `${cachePrefix}account:${data.address}:rewards`,
    onLoading: rewardsLoading,
    onSuccess: rewardsSuccess,
    onError: rewardsError,
  })}

  <!-- Transaction History -->
  {@render matchPromiseWithCache(data.transactions, {
    cacheKey: `${cachePrefix}account:${data.address}:transactions`,
    onLoading: timelineLoading,
    onSuccess: timelineSuccess,
    onError: timelineError,
  })}
</div>
