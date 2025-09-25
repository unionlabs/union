<script lang="ts">
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import * as O from "effect/Option"
import type { Snippet } from "svelte"

interface Props {
  label: string
  value: O.Option<string | number>
  subtitle: O.Option<string>
  icon?: Snippet
  loading?: boolean
  clickable?: boolean
  indicator?: Snippet
  class?: string
  onclick?: () => void
}

let {
  label,
  value = O.none(),
  subtitle = O.none(),
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
        <div class="text-xs text-zinc-500 font-medium tracking-wider">
          {label}
        </div>
      </div>
      {#if loading || O.isNone(value)}
        <div class="text-sm sm:text-lg font-semibold tabular-nums">
          <Skeleton class="h-5 sm:h-6 w-20" />
        </div>
      {:else if O.isSome(subtitle)}
        <div class="flex items-baseline gap-1.5">
          <span class="text-sm sm:text-lg font-semibold text-zinc-100 tabular-nums">
            {O.getOrElse(value, () => "")}
          </span>
          <span class="text-xs text-zinc-500">
            {O.getOrElse(subtitle, () => "")}
          </span>
        </div>
      {:else}
        <div class="text-sm sm:text-lg font-semibold text-zinc-100 tabular-nums">
          {O.getOrElse(value, () => "")}
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
      {#if loading || O.isNone(value)}
        <div class="text-sm sm:text-lg font-semibold tabular-nums">
          <Skeleton class="h-5 sm:h-6 w-20" />
        </div>
      {:else if O.isSome(subtitle)}
        <div class="flex items-baseline gap-1.5">
          <span class="text-sm sm:text-lg font-semibold text-zinc-100 tabular-nums">
            {O.getOrElse(value, () => "")}
          </span>
          <span class="text-xs text-zinc-500">
            {O.getOrElse(subtitle, () => "")}
          </span>
        </div>
      {:else}
        <div class="text-sm sm:text-lg font-semibold text-zinc-100 tabular-nums">
          {O.getOrElse(value, () => "")}
        </div>
      {/if}
    </div>
  </div>
{/if}
