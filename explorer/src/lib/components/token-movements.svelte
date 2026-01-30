<script lang="ts">
import CornerMarks from "$lib/components/corner-marks.svelte"
import * as Collapsible from "$lib/components/ui/collapsible/index.js"
import { formatAmount, truncateAddress } from "$lib/utils/format"
import {
  extractTokenMovements,
  formatDenom,
  getMovementColor,
  type TokenMovementSummary,
} from "$lib/utils/tokens"
import { urls } from "$lib/utils/urls"
import ArrowDownLeftIcon from "@lucide/svelte/icons/arrow-down-left"
import ArrowUpRightIcon from "@lucide/svelte/icons/arrow-up-right"
import ChevronDownIcon from "@lucide/svelte/icons/chevron-down"
import FlameIcon from "@lucide/svelte/icons/flame"
import SparklesIcon from "@lucide/svelte/icons/sparkles"

type TxEvent = {
  type: string
  attributes: Array<{ key: string; value: string; index?: boolean }>
}

interface Props {
  events: TxEvent[]
  messageCount?: number
}

let { events, messageCount = 1 }: Props = $props()

const summary = $derived(extractTokenMovements(events))

// Check if there are any meaningful movements
const hasMovements = $derived(summary.movements.length > 0)

// Get unique denoms for the summary
const uniqueDenoms = $derived([...summary.netFlow.keys()])

// Format bigint to string for display
function formatBigInt(value: bigint, decimals: number = 18): string {
  return formatAmount(value.toString(), decimals)
}

// Get icon component for direction
function getIcon(direction: string) {
  switch (direction) {
    case "sent":
      return ArrowUpRightIcon
    case "received":
      return ArrowDownLeftIcon
    case "burned":
      return FlameIcon
    case "minted":
      return SparklesIcon
    default:
      return ArrowUpRightIcon
  }
}
</script>

{#if hasMovements}
  <div class="relative border border-border">
    <CornerMarks />

    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
      <div class="flex items-center gap-3">
        <span class="text-[10px] font-mono text-muted-foreground">TKN</span>
        <span class="text-xs font-medium uppercase tracking-wider">Token Movements</span>
        <span class="text-xs text-muted-foreground">({summary.movements.length} transfers)</span>
      </div>
    </div>

    <!-- Net Flow Summary -->
    {#if uniqueDenoms.length > 0}
      <div class="p-4 border-b border-border bg-muted/10">
        <div class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground mb-3">
          Net Flow
        </div>
        <div class="flex flex-wrap gap-3">
          {#each uniqueDenoms as denom}
            {@const net = summary.netFlow.get(denom) ?? 0n}
            {@const isPositive = net > 0n}
            {@const isNegative = net < 0n}
            <div class="flex items-center gap-2 px-3 py-2 border border-border bg-background">
              {#if isPositive}
                <ArrowDownLeftIcon class="h-4 w-4 text-success" />
                <span class="font-mono text-sm text-success">+{formatBigInt(net)}</span>
              {:else if isNegative}
                <ArrowUpRightIcon class="h-4 w-4 text-destructive" />
                <span class="font-mono text-sm text-destructive">{formatBigInt(net)}</span>
              {:else}
                <span class="font-mono text-sm text-muted-foreground">0</span>
              {/if}
              <span class="text-xs text-muted-foreground font-mono">{formatDenom(denom)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Totals -->
    <div class="grid grid-cols-2 gap-px bg-border">
      <!-- Sent -->
      <div class="bg-background p-4">
        <div class="flex items-center gap-2 mb-2">
          <ArrowUpRightIcon class="h-4 w-4 text-destructive" />
          <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground"
          >Total Sent</span>
        </div>
        {#if summary.totals.sent.size > 0}
          <div class="space-y-1">
            {#each [...summary.totals.sent.entries()] as [denom, amount]}
              <div class="flex items-baseline gap-2">
                <span class="font-mono text-sm text-destructive">{formatBigInt(amount)}</span>
                <span class="text-xs text-muted-foreground font-mono">{formatDenom(denom)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <span class="text-sm text-muted-foreground">-</span>
        {/if}
      </div>

      <!-- Received -->
      <div class="bg-background p-4">
        <div class="flex items-center gap-2 mb-2">
          <ArrowDownLeftIcon class="h-4 w-4 text-success" />
          <span class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground"
          >Total Received</span>
        </div>
        {#if summary.totals.received.size > 0}
          <div class="space-y-1">
            {#each [...summary.totals.received.entries()] as [denom, amount]}
              <div class="flex items-baseline gap-2">
                <span class="font-mono text-sm text-success">{formatBigInt(amount)}</span>
                <span class="text-xs text-muted-foreground font-mono">{formatDenom(denom)}</span>
              </div>
            {/each}
          </div>
        {:else}
          <span class="text-sm text-muted-foreground">-</span>
        {/if}
      </div>
    </div>

    <!-- Per-Message Breakdown (collapsible) -->
    {#if messageCount > 1 && summary.perMessage.size > 0}
      <Collapsible.Root>
        <Collapsible.Trigger class="w-full">
          <div class="flex items-center gap-2 px-4 py-2.5 border-t border-border hover:bg-muted/20 transition-colors">
            <ChevronDownIcon
              class="h-3.5 w-3.5 text-muted-foreground shrink-0 transition-transform [[data-state=open]_&]:rotate-180"
            />
            <span class="text-[11px] font-medium uppercase tracking-wider text-muted-foreground"
            >Per-Message Breakdown</span>
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border">
            {#each [...summary.perMessage.entries()].sort((a, b) => a[0] - b[0]) as
              [msgIdx, movements]
            }
              <div class="border-b border-border/50 last:border-b-0">
                <div class="px-4 py-2 bg-muted/10">
                  <span class="text-[10px] font-mono text-muted-foreground">Message {
                      msgIdx + 1
                    }</span>
                </div>
                <div class="divide-y divide-border/30">
                  {#each movements as movement}
                    {@const Icon = getIcon(movement.direction)}
                    {@const colorClass = getMovementColor(movement.direction)}
                    <div class="flex items-center gap-3 px-4 py-2 hover:bg-muted/5">
                      <Icon class="h-3.5 w-3.5 {colorClass} shrink-0" />
                      <span class="text-[10px] font-mono uppercase w-16 shrink-0 {colorClass}">{
                        movement.direction
                      }</span>
                      <a
                        href={urls.account(movement.address)}
                        class="text-xs font-mono text-info hover:underline truncate"
                      >
                        {truncateAddress(movement.address, 8)}
                      </a>
                      <div class="flex items-baseline gap-1.5 ml-auto shrink-0">
                        <span class="font-mono text-sm {colorClass}">{
                          formatAmount(movement.amount, 18)
                        }</span>
                        <span class="text-[10px] text-muted-foreground font-mono">{
                          formatDenom(movement.denom)
                        }</span>
                      </div>
                      {#if movement.type === "cw20"}
                        <span
                          class="text-[9px] px-1.5 py-0.5 bg-purple-500/10 text-purple-400 border border-purple-500/30"
                        >CW20</span>
                      {/if}
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </Collapsible.Content>
      </Collapsible.Root>
    {/if}

    <!-- All Movements (if single message or collapsed per-message) -->
    {#if messageCount <= 1 && summary.movements.length > 0}
      <Collapsible.Root>
        <Collapsible.Trigger class="w-full">
          <div class="flex items-center gap-2 px-4 py-2.5 border-t border-border hover:bg-muted/20 transition-colors">
            <ChevronDownIcon
              class="h-3.5 w-3.5 text-muted-foreground shrink-0 transition-transform [[data-state=open]_&]:rotate-180"
            />
            <span class="text-[11px] font-medium uppercase tracking-wider text-muted-foreground"
            >All Transfers</span>
            <span class="text-[10px] text-muted-foreground/70">({summary.movements.length})</span>
          </div>
        </Collapsible.Trigger>
        <Collapsible.Content>
          <div class="border-t border-border divide-y divide-border/30">
            {#each summary.movements as movement, i}
              {@const Icon = getIcon(movement.direction)}
              {@const colorClass = getMovementColor(movement.direction)}
              <div class="flex items-center gap-3 px-4 py-2 hover:bg-muted/5">
                <span class="text-[10px] font-mono text-muted-foreground/50 w-4">{i}</span>
                <Icon class="h-3.5 w-3.5 {colorClass} shrink-0" />
                <span class="text-[10px] font-mono uppercase w-16 shrink-0 {colorClass}">{
                  movement.direction
                }</span>
                <a
                  href={urls.account(movement.address)}
                  class="text-xs font-mono text-info hover:underline truncate"
                >
                  {truncateAddress(movement.address, 8)}
                </a>
                <div class="flex items-baseline gap-1.5 ml-auto shrink-0">
                  <span class="font-mono text-sm {colorClass}">{
                    formatAmount(movement.amount, 18)
                  }</span>
                  <span class="text-[10px] text-muted-foreground font-mono">{
                    formatDenom(movement.denom)
                  }</span>
                </div>
                {#if movement.type === "cw20"}
                  <span
                    class="text-[9px] px-1.5 py-0.5 bg-purple-500/10 text-purple-400 border border-purple-500/30"
                  >CW20</span>
                {/if}
              </div>
            {/each}
          </div>
        </Collapsible.Content>
      </Collapsible.Root>
    {/if}
  </div>
{/if}
