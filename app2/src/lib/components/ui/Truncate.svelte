<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import SharpContentCopyIcon from "$lib/components/icons/SharpContentCopyIcon.svelte"
import SharpCheckIcon from "$lib/components/icons/SharpCheckIcon.svelte"
import { page } from "$app/state"

interface Props extends HTMLAttributes<HTMLDivElement> {
  value: string
  maxLength?: number
  class?: string
  showCopy?: boolean
}

const { value, maxLength = 8, class: className = "", showCopy = true, ...rest }: Props = $props()

const shouldTruncate = $derived(value.length > maxLength)
const displayValue = $derived(
  shouldTruncate
    ? `${value.slice(0, Math.floor(maxLength / 2))}…${value.slice(-Math.floor(maxLength / 2))}`
    : value
)

let showCopied = $state(false)
let timeout: ReturnType<typeof setTimeout>

async function copyToClipboard() {
  await navigator.clipboard.writeText(value)
  showCopied = true

  clearTimeout(timeout)
  timeout = setTimeout(() => {
    showCopied = false
  }, 1000)
}
</script>

<div class={cn("group flex items-center gap-1", className)} {...rest}>
  <span title={shouldTruncate ? value : undefined}>
    {displayValue}
  </span>
  {#if showCopy}
    <button
      type="button"
      class="p-1 text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity"
      onclick={(e) => {
        e.stopPropagation();
        copyToClipboard();
      }}
    >
      {#if showCopied}
        <SharpCheckIcon class="text-green-500" />
      {:else}
        <SharpContentCopyIcon />
      {/if}
    </button>
  {/if}
</div>
