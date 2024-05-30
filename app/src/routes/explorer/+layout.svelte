<script lang="ts">
import { onMount } from "svelte"
import Menu from "./(components)/menu.svelte"
import type { LayoutData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Resizable from "$lib/components/ui/resizable"
import GripVerticalIcon from "virtual:icons/tabler/grip-vertical"
import { ScrollArea } from "$lib/components/ui/scroll-area/index.ts"

export let data: LayoutData

let windowSize = { width: window.innerWidth, height: window.innerHeight }

const handleResize = () => {
  requestAnimationFrame(() => {
    windowSize = { width: window.innerWidth, height: window.innerHeight }
  })
}

onMount(() => {
  window.addEventListener("resize", handleResize)
  return () => {
    window.removeEventListener("resize", handleResize)
  }
})

let isCollapsed = false
let leftPane: Resizable.PaneAPI
$: [leftSize, rightSize] = [12, 88]

$: if (windowSize.width < 900) leftPane.collapse()

const onLayoutChange: Resizable.PaneGroupProps["onLayoutChange"] = sizes => {
  document.cookie = `PaneForge:layout=${JSON.stringify(sizes)}`
}

const onCollapse: Resizable.PaneProps["onExpand"] = () => {
  isCollapsed = true
  document.cookie = `PaneForge:collapsed=${true}`
}

const onExpand: Resizable.PaneProps["onExpand"] = () => {
  isCollapsed = false
  document.cookie = `PaneForge:collapsed=${false}`
}
</script>

<main class="flex flex-row flex-1 overflow-y-hidden resize">
  <Resizable.PaneGroup direction="horizontal" class="w-full rounded-lg bg-re" {onLayoutChange}>
    <Resizable.Pane
      {onExpand}
      {onCollapse}
      maxSize={12}
      minSize={12}
      collapsible={true}
      collapsedSize={4.5}
      bind:pane={leftPane}
      defaultSize={leftSize}
      class={cn(
        isCollapsed ? 'min-w-13 max-w-13' : 'min-w-[160px] max-w-[180px]',
        'w-full border-r border-solid border-r-secondary',
      )}
    >
      <Menu tableRoutes={data.tables} {isCollapsed} />
    </Resizable.Pane>
    <Resizable.Handle
      withHandle
      class="relative flex w-4 max-w-4 items-center justify-center bg-background"
    >
      <div class="h-full w-12 items-center justify-center rounded-sm border bg-muted">
        <GripVerticalIcon />
      </div>
    </Resizable.Handle>
    <Resizable.Pane defaultSize={rightSize} class="rounded-lg p-0">
      <ScrollArea orientation="both" class="size-full flex-1">
        <slot />
      </ScrollArea>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</main>
