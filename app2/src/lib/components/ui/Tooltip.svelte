<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  trigger: Snippet
  content: Snippet
  class?: string
}

let { trigger, content, class: className = "", ...rest }: Props = $props()

let tooltipElement: HTMLDivElement
let triggerElement: HTMLDivElement
let isVisible = $state(false)

function showTooltip(e: MouseEvent) {
  updatePosition(e)
  isVisible = true
}

function hideTooltip() {
  isVisible = false
}

function updatePosition(e?: MouseEvent) {
  if (!(tooltipElement && e)) return

  const tooltipRect = tooltipElement.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  // Start with cursor position relative to viewport
  let x = e.clientX
  let y = e.clientY

  // Add offset to position tooltip near cursor
  const OFFSET_X = 10
  const OFFSET_Y = 10
  x += OFFSET_X
  y += OFFSET_Y

  // Check if tooltip would go off right edge
  if (x + tooltipRect.width > viewportWidth) {
    // Position to left of cursor instead
    x = e.clientX - tooltipRect.width - OFFSET_X
  }

  // Check if tooltip would go off bottom edge
  if (y + tooltipRect.height > viewportHeight) {
    // Position above cursor instead
    y = e.clientY - tooltipRect.height - OFFSET_Y
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
  onmousemove={updatePosition}
>
  {@render trigger()}
</div>

{#if isVisible}
<div 
  bind:this={tooltipElement}
  class={tooltipClasses}
  {...rest}
>
  {@render content()}
</div>
{/if}
