<script lang="ts">
import type { Snippet } from "svelte"

interface Props {
  label: string
  value?: string | number | undefined
  subtitle?: string | undefined
  icon?: Snippet
  loading?: boolean
  clickable?: boolean
  indicator?: Snippet
  class?: string
  onclick?: () => void
}

let {
  label,
  value,
  subtitle,
  icon,
  loading = false,
  clickable = false,
  indicator,
  class: className = "",
  onclick,
}: Props = $props()

const baseClass = "rounded-lg bg-zinc-900 border border-zinc-800/50 p-3"
const interactiveClass = clickable ? "cursor-pointer text-left" : ""
const combinedClass = `${baseClass} ${interactiveClass} ${className}`
</script>

{#if clickable}
  <button 
    {onclick}
    class="{combinedClass} relative"
  >
    {#if indicator}
      <div class="absolute top-2 right-2">
        {@render indicator()}
      </div>
    {/if}
    
    <div class="flex flex-col gap-1.5">
      <div class="flex items-center gap-1.5">
        {#if icon}
          {@render icon()}
        {/if}
        <div class="text-xs text-zinc-500 font-medium uppercase tracking-wider">
          {label}
        </div>
      </div>
      {#if loading || value === undefined}
        <div class="h-6 w-14 bg-zinc-800/50 rounded animate-pulse"></div>
      {:else if subtitle}
        <div class="flex items-baseline gap-1.5">
          <span class="text-lg font-semibold text-zinc-100 tabular-nums">
            {value}
          </span>
          <span class="text-xs text-zinc-500">
            {subtitle}
          </span>
        </div>
      {:else}
        <div class="text-lg font-semibold text-zinc-100 tabular-nums">
          {value}
        </div>
      {/if}
    </div>
  </button>
{:else}
  <div class="{combinedClass} relative">
    {#if indicator}
      <div class="absolute top-2 right-2">
        {@render indicator()}
      </div>
    {/if}
    
    <div class="flex flex-col gap-1.5">
      <div class="flex items-center gap-1.5">
        {#if icon}
          {@render icon()}
        {/if}
        <div class="text-xs text-zinc-500 font-medium uppercase tracking-wider">
          {label}
        </div>
      </div>
      {#if loading || value === undefined}
        <div class="h-6 w-14 bg-zinc-800/50 rounded animate-pulse"></div>
      {:else if subtitle}
        <div class="flex items-baseline gap-1.5">
          <span class="text-lg font-semibold text-zinc-100 tabular-nums">
            {value}
          </span>
          <span class="text-xs text-zinc-500">
            {subtitle}
          </span>
        </div>
      {:else}
        <div class="text-lg font-semibold text-zinc-100 tabular-nums">
          {value}
        </div>
      {/if}
    </div>
  </div>
{/if}
