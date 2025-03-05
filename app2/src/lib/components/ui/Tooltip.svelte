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
let position = $state({
  placement: "top",
  shift: 0
})

function updatePosition() {
  if (!(tooltipElement && triggerElement)) return

  const triggerRect = triggerElement.getBoundingClientRect()
  const tooltipRect = tooltipElement.getBoundingClientRect()
  const viewportWidth = window.innerWidth

  let placement = "top"
  let shift = 0

  // Check if there's enough space above
  if (triggerRect.top - tooltipRect.height < 0) {
    placement = "bottom"
  }

  // Calculate horizontal overflow
  const centerX = triggerRect.left + triggerRect.width / 2
  const halfWidth = tooltipRect.width / 2

  // Check left edge
  if (centerX - halfWidth < 0) {
    shift = -(centerX - halfWidth)
  }
  // Check right edge
  else if (centerX + halfWidth > viewportWidth) {
    shift = -(centerX + halfWidth - viewportWidth)
  }

  position = { placement, shift }
}

$effect(() => {
  const observer = new ResizeObserver(updatePosition)
  if (tooltipElement) {
    observer.observe(tooltipElement)
  }
  window.addEventListener("scroll", updatePosition)
  window.addEventListener("resize", updatePosition)

  return () => {
    observer.disconnect()
    window.removeEventListener("scroll", updatePosition)
    window.removeEventListener("resize", updatePosition)
  }
})

const tooltipClasses = $derived(
  cn(
    "z-40 overflow-hidden border border-1 border-zinc-800 bg-black p-2 rounded shadow-md",
    "invisible group-hover:visible absolute left-1/2",
    position.placement === "top" ? "bottom-full" : "top-full",
    className
  )
)

const tooltipStyle = $derived(`transform: translateX(calc(-50% + ${position.shift}px))`)
</script>

<div class="group relative inline-block" bind:this={triggerElement}>
  {@render trigger()}
  <div 
    bind:this={tooltipElement}
    class={tooltipClasses}
    style={tooltipStyle}
    {...rest}
  >
    {@render content()}
  </div>
</div>
