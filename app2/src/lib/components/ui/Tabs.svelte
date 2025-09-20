<script lang="ts">
import { cn } from "$lib/utils"

interface TabItem {
  id: string
  label: string
}

interface Props {
  items: TabItem[]
  activeId: string
  onTabChange: (id: string) => void
  class?: string
}

const { items, activeId, onTabChange, class: className }: Props = $props()

let wrapperRef: HTMLElement
let itemRefs: HTMLElement[] = []
let activeTabStyle = $state({ width: 0, height: 0, left: 0, top: 0 })

const updateActiveStyle = () => {
  const activeIndex = items.findIndex(item => item.id === activeId)
  const activeElement = itemRefs[activeIndex]

  if (activeElement) {
    activeTabStyle = {
      width: activeElement.offsetWidth,
      height: activeElement.offsetHeight,
      left: activeElement.offsetLeft,
      top: activeElement.offsetTop,
    }
  }
}

// Update style when active tab changes or when component mounts
$effect(() => {
  if (activeId && itemRefs.length > 0) {
    updateActiveStyle()
  }
})

// Update style when items change (for dynamic content)
$effect(() => {
  if (items && itemRefs.length === items.length) {
    setTimeout(updateActiveStyle, 0) // Allow DOM to update first
  }
})
</script>

<div
  bind:this={wrapperRef}
  class={cn("flex gap-1 relative", className)}
>
  <!-- Sliding background indicator -->
  <div
    class="absolute bg-zinc-800/90 border border-zinc-700/50 rounded-md transition-all duration-300 z-0 shadow-sm"
    style:width="{activeTabStyle.width}px"
    style:height="{activeTabStyle.height}px"
    style:left="{activeTabStyle.left}px"
    style:top="{activeTabStyle.top}px"
  >
  </div>

  <!-- Tab buttons -->
  {#each items as item, index (item.id)}
    <button
      bind:this={itemRefs[index]}
      onclick={() => onTabChange(item.id)}
      class={cn(
        "px-3 sm:px-4 py-2 sm:py-2 text-sm font-medium rounded-md transition-colors relative cursor-pointer z-10",
        activeId === item.id
          ? "text-white"
          : "text-zinc-500 hover:text-zinc-300",
      )}
    >
      {item.label}
    </button>
  {/each}
</div>
