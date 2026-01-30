<script lang="ts" module>
import type { Component } from "svelte"
import { copyToClipboard } from "$lib/utils/clipboard"
import Copy from "@lucide/svelte/icons/copy"
import Check from "@lucide/svelte/icons/check"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import CornerMarks from "$lib/components/corner-marks.svelte"

export { dataRow, sectionHeader, statCard, statCardLoading }
</script>

{#snippet dataRow(label: string, value: string | number | undefined | null, options?: { mono?: boolean, link?: string, copy?: boolean })}
  <div class="flex items-center justify-between py-2.5 px-4 hover:bg-muted/30 group border-b border-border last:border-b-0">
    <span class="text-xs font-mono uppercase tracking-wider text-muted-foreground">{label}</span>
    <div class="flex items-center gap-2 min-w-0 flex-1 justify-end">
      {#if value !== undefined && value !== null && value !== ""}
        {#if options?.link}
          <a href={options.link} class="font-mono text-sm truncate hover:underline max-w-[400px]">{value}</a>
        {:else}
          <span class="text-sm truncate max-w-[400px] {options?.mono ? 'font-mono' : ''}">{value}</span>
        {/if}
        {#if options?.copy && typeof value === 'string'}
          {@const copyValue = value}
          <button
            onclick={(e) => {
              const btn = e.currentTarget as HTMLButtonElement
              copyToClipboard(copyValue, () => {
                btn.dataset.copied = "true"
                setTimeout(() => btn.dataset.copied = "false", 2000)
              })
            }}
            class="opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-muted data-[copied=true]:opacity-100"
          >
            <Copy class="h-3 w-3 text-muted-foreground [[data-copied=true]_&]:hidden" />
            <Check class="h-3 w-3 text-green-500 hidden [[data-copied=true]_&]:block" />
          </button>
        {/if}
      {:else}
        <span class="text-muted-foreground text-sm">-</span>
      {/if}
    </div>
  </div>
{/snippet}

{#snippet sectionHeader(title: string, num?: string, icon?: Component, count?: number)}
  <div class="flex items-center gap-3 px-4 py-3 border-b border-border bg-muted/20">
    {#if num}
      <span class="text-[10px] font-mono text-muted-foreground">{num}</span>
    {/if}
    {#if icon}
      <svelte:component this={icon} class="h-4 w-4 text-muted-foreground" />
    {/if}
    <span class="text-xs font-medium uppercase tracking-wider">{title}</span>
    {#if count !== undefined}
      <span class="text-xs text-muted-foreground">({count})</span>
    {/if}
  </div>
{/snippet}

{#snippet statCard(icon: Component, label: string, value: string, sub?: string)}
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
