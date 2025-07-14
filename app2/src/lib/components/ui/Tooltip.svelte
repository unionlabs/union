<script lang="ts">
import { cn } from "$lib/utils"
import { Match } from "effect"
import type { Snippet } from "svelte"
import type { HTMLAttributes } from "svelte/elements"
import { scale } from "svelte/transition"

type Props = HTMLAttributes<HTMLDivElement> & {
  trigger: Snippet
  content: Snippet
  title?: string
  class?: string
  delay?: "quick" | undefined
}

let { trigger, content, title, class: className = "", delay, ...rest }: Props = $props()

let tooltipElement = $state<HTMLDivElement>()
let isVisible = $state(false)
let isHoveringTooltip = $state(false)
let isHoveringTrigger = $state(false)
let isMobile = $state(false)

let lastMouseEvent: MouseEvent | undefined
let tooltipReady = $state(false)
let showTimeout: number | undefined
let mobileHideTimeout: number | undefined

let [enterDelay, leaveDelay] = Match.value(delay).pipe(
  Match.when(Match.undefined, () => [750, 250]),
  Match.when("quick", () => [100, 100]),
  Match.exhaustive,
)

$effect(() => {
  if (isVisible && tooltipElement && lastMouseEvent) {
    tooltipReady = true
    updatePosition(lastMouseEvent)
  } else {
    tooltipReady = false
  }
})

$effect(() => {
  // More conservative mobile detection - only treat as mobile if it's clearly a mobile device
  function updateMobileState() {
    isMobile = /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
      || (window.matchMedia?.("(pointer: coarse)").matches
        && window.matchMedia?.("(hover: none)").matches)
  }

  // Initial detection
  updateMobileState()

  // Listen for window resize/orientation changes
  window.addEventListener("resize", updateMobileState)
  window.addEventListener("orientationchange", updateMobileState)

  return () => {
    window.removeEventListener("resize", updateMobileState)
    window.removeEventListener("orientationchange", updateMobileState)
  }
})

$effect(() => {
  if (!isMobile || !isVisible) {
    return
  }

  function handleClickOutside(e: Event) {
    if (tooltipElement && !tooltipElement.contains(e.target as Node)) {
      const triggerElement = (e.target as Element)?.closest("[data-tooltip-trigger]")
      if (!triggerElement) {
        isVisible = false
        hideTooltip()
      }
    }
  }

  document.addEventListener("touchstart", handleClickOutside)

  return () => {
    document.removeEventListener("touchstart", handleClickOutside)
  }
})

function hideTooltip() {
  // Clear any pending show timeout
  if (showTimeout) {
    clearTimeout(showTimeout)
    showTimeout = undefined
  }

  // Clear any pending mobile hide timeout
  if (mobileHideTimeout) {
    clearTimeout(mobileHideTimeout)
    mobileHideTimeout = undefined
  }
}

function onTooltipEnter() {
  if (isMobile) {
    return // Skip mouse events on mobile
  }
  isHoveringTooltip = true
}

function onTooltipLeave() {
  if (isMobile) {
    return // Skip mouse events on mobile
  }

  isHoveringTooltip = false
  window.setTimeout(() => {
    if (!(isHoveringTrigger || isHoveringTooltip)) {
      isVisible = false
    }
  }, leaveDelay)
}

function onTriggerEnter(e: MouseEvent) {
  if (isMobile) {
    return // Skip mouse events on mobile
  }

  isHoveringTrigger = true
  lastMouseEvent = e
  showTimeout = window.setTimeout(() => {
    if (isHoveringTrigger) {
      isVisible = true
    }
  }, enterDelay)
}

function onTriggerLeave() {
  if (isMobile) {
    return // Skip mouse events on mobile
  }

  isHoveringTrigger = false
  window.setTimeout(() => {
    if (!isHoveringTooltip) {
      isVisible = false
    }
  }, leaveDelay)
}

function onTriggerMove(e: MouseEvent) {
  if (isMobile) {
    return // Skip mouse events on mobile
  }

  if (isVisible && tooltipReady) {
    lastMouseEvent = e
    updatePosition(e)
  } else if (isHoveringTrigger) {
    lastMouseEvent = e
  }
}

function onTouchStart(e: TouchEvent) {
  if (!isMobile) {
    return
  }

  e.preventDefault()

  // Get touch position and convert to mouse-like event
  const touch = e.touches[0]
  const mouseEvent = {
    clientX: touch.clientX,
    clientY: touch.clientY,
  } as MouseEvent

  if (isVisible) {
    // If tooltip is already visible, hide it
    isVisible = false
    hideTooltip()
  } else {
    // Show tooltip immediately on mobile (no delay)
    lastMouseEvent = mouseEvent
    isVisible = true

    // No auto-hide on mobile - let user dismiss manually
  }
}

function updatePosition(e?: MouseEvent) {
  if (!(tooltipElement && e && tooltipReady)) {
    return
  }

  const tooltipRect = tooltipElement.getBoundingClientRect()
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight
  const VERTICAL_OFFSET = 15
  // Use smaller padding on mobile since screen space is limited
  const HORIZONTAL_PADDING = isMobile ? 12 : 20
  const VERTICAL_PADDING = isMobile ? 12 : 20

  // Start with cursor position relative to viewport
  let x = e.clientX - tooltipRect.width / 2 // Center horizontally on cursor
  let y = e.clientY

  // Handle horizontal overflow with more padding
  if (x < HORIZONTAL_PADDING) {
    // Too far left, align to left edge with padding
    x = HORIZONTAL_PADDING
  } else if (x + tooltipRect.width > viewportWidth - HORIZONTAL_PADDING) {
    // Too far right, align to right edge with padding
    x = viewportWidth - tooltipRect.width - HORIZONTAL_PADDING
  }

  // Handle vertical positioning with better overflow detection
  const spaceAbove = e.clientY - VERTICAL_OFFSET
  const spaceBelow = viewportHeight - e.clientY - VERTICAL_OFFSET
  const tooltipHeight = tooltipRect.height

  if (tooltipHeight <= spaceAbove - VERTICAL_PADDING) {
    // Enough space above, position above cursor
    y = e.clientY - tooltipHeight - VERTICAL_OFFSET
  } else if (tooltipHeight <= spaceBelow - VERTICAL_PADDING) {
    // Not enough space above but enough below, position below cursor
    y = e.clientY + VERTICAL_OFFSET
  } else {
    // Not enough space above or below, position where there's more space
    if (spaceAbove > spaceBelow) {
      // More space above, position at top with padding
      y = VERTICAL_PADDING
    } else {
      // More space below, position to fit in remaining space
      y = Math.max(VERTICAL_PADDING, viewportHeight - tooltipHeight - VERTICAL_PADDING)
    }
  }

  // Final bounds check to ensure tooltip stays within viewport with padding
  y = Math.max(VERTICAL_PADDING, Math.min(viewportHeight - tooltipHeight - VERTICAL_PADDING, y))

  tooltipElement.style.left = `${x}px`
  tooltipElement.style.top = `${y}px`
}

const tooltipClasses = $derived(
  cn(
    "fixed z-[999999] cursor-default border border-1 border-zinc-700 bg-zinc-800 p-2 rounded shadow-md max-w-md max-h-96 overflow-auto",
    isVisible && "opacity-100 visible delay-600",
    isVisible ? "scale-100" : "scale-95",
    className,
  ),
)
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="cursor-pointer"
  data-tooltip-trigger
  onmouseenter={onTriggerEnter}
  onmouseleave={onTriggerLeave}
  onmousemove={onTriggerMove}
  ontouchstart={onTouchStart}
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
      opacity: 0,
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
