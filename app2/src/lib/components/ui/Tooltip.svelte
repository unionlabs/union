<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"
import { scale } from "svelte/transition"

type Props = HTMLAttributes<HTMLDivElement> & {
  trigger: Snippet
  content: Snippet
  class?: string
}

let { trigger, content, class: className = "", ...rest }: Props = $props()

let tooltipElement: HTMLDivElement
let triggerElement: HTMLDivElement
let isVisible = $state(false)
let isHoveringTooltip = $state(false)

let lastMouseEvent: MouseEvent | undefined
let tooltipReady = $state(false)
let showTimeout: number | undefined

function showTooltip(e: MouseEvent) {
  lastMouseEvent = e
  showTimeout = window.setTimeout(() => {
    isVisible = true
  }, 1000)
}

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

  // Only hide if we're not hovering the tooltip
  if (!isHoveringTooltip) {
    isVisible = false
  }
}

function onTooltipEnter() {
  isHoveringTooltip = true
}

function onTooltipLeave() {
  isHoveringTooltip = false
  isVisible = false
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
    "fixed z-40 overflow-hidden border border-1 border-zinc-800 bg-black p-2 rounded shadow-md",
    isVisible && "opacity-100 visible delay-600",
    isVisible ? "scale-100" : "scale-95",
    className
  )
)

// const tooltipStyle = $derived(`left: ${position.x}px; top: ${position.y}px`)
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
  class="inline-block" 
  bind:this={triggerElement}
  onmouseenter={showTooltip}
  onmouseleave={hideTooltip}
>
  {@render trigger()}
</div>

{#if isVisible}
<div 
  bind:this={tooltipElement}
  class={tooltipClasses}
  onmouseenter={onTooltipEnter}
  onmouseleave={onTooltipLeave}
  transition:scale|local={{
    duration: 150,
    start: 0.95,
    opacity: 0
  }}
  {...rest}
>
  {@render content()}
</div>
{/if}
