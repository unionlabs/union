<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"
import { scale } from "svelte/transition"

type Props = HTMLAttributes<HTMLDivElement> & {
  trigger: Snippet
  content: Snippet
  title?: string
  class?: string
}

let { trigger, content, title, class: className = "", ...rest }: Props = $props()

let tooltipElement: HTMLDivElement
let isVisible = $state(false)
let isHoveringTooltip = $state(false)
let isHoveringTrigger = $state(false)

let lastMouseEvent: MouseEvent | undefined
let tooltipReady = $state(false)
let showTimeout: number | undefined

$effect(() => {
  if (isVisible && tooltipElement && lastMouseEvent) {
    tooltipReady = true
    updatePosition(lastMouseEvent)
  } else {
    tooltipReady = false
  }
})

function hideTooltip() {
  // Clear any pending show timeout
  if (showTimeout) {
    clearTimeout(showTimeout)
    showTimeout = undefined
  }
}

function onTooltipEnter() {
  isHoveringTooltip = true
}

function onTooltipLeave() {
  isHoveringTooltip = false
  window.setTimeout(() => {
    if (!(isHoveringTrigger || isHoveringTooltip)) {
      isVisible = false
    }
  }, 200)
}

function onTriggerEnter(e: MouseEvent) {
  isHoveringTrigger = true
  lastMouseEvent = e
  showTimeout = window.setTimeout(() => {
    if (isHoveringTrigger) {
      isVisible = true
    }
  }, 750)
}

function onTriggerLeave() {
  isHoveringTrigger = false
  window.setTimeout(() => {
    if (!isHoveringTooltip) {
      isVisible = false
    }
  }, 200)
}

function updatePosition(e?: MouseEvent) {
  if (!(tooltipElement && e && tooltipReady)) return

  const tooltipRect = tooltipElement.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const VERTICAL_OFFSET = 15

  // Start with cursor position relative to viewport
  let x = e.clientX - tooltipRect.width / 2 // Center horizontally on cursor
  let y = e.clientY

  // Ensure tooltip stays within horizontal bounds
  x = Math.max(10, Math.min(viewportWidth - tooltipRect.width - 10, x))

  // Check if there's room above cursor
  if (e.clientY - tooltipRect.height - VERTICAL_OFFSET > 0) {
    // Position above cursor
    y = e.clientY - tooltipRect.height - VERTICAL_OFFSET
  } else {
    // Position below cursor
    y = e.clientY + VERTICAL_OFFSET
  }

  tooltipElement.style.left = `${x}px`
  tooltipElement.style.top = `${y}px`
}

const tooltipClasses = $derived(
  cn(
    "fixed z-40 cursor-default overflow-visible border border-1 border-zinc-700 bg-zinc-800 p-2 rounded shadow-md",
    isVisible && "opacity-100 visible delay-600",
    isVisible ? "scale-100" : "scale-95",
    className
  )
)
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
  class="inline-block cursor-pointer" 
  onmouseenter={onTriggerEnter}
  onmouseleave={onTriggerLeave}
>
  {@render trigger()}
</div>

{#if isVisible}
<div 
  bind:this={tooltipElement}
  class={tooltipClasses}
  onmouseenter={onTooltipEnter}
  onmouseleave={onTooltipLeave}
  onclick={(e) => e.stopPropagation()}
  onmousedown={(e) => e.stopPropagation()}
  transition:scale|local={{
    duration: 150,
    start: 0.95,
    opacity: 0
  }}
  {...rest}
>
  <div class="tooltip-content text-sm flex flex-col gap-4 text-left">
    {#if title}
      <section class="flex justify-between items-center">
        <h2 class="text-white font-bold text-lg">{title}</h2>
      </section>
    {/if}
    {@render content()}
  </div>
</div>
{/if}

<style global>
  .tooltip-content * {
    pointer-events: auto !important;
  }
</style>
