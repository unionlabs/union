<script lang="ts">
import Menu from "./components/menu.svelte"
import type { LayoutData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Resizable from "$lib/components/ui/resizable"
import GripVerticalIcon from "virtual:icons/tabler/grip-vertical"
import { ScrollArea } from "$lib/components/ui/scroll-area/index.ts"

export let data: LayoutData

let isCollapsed = false
$: [leftSize, rightSize] = [12, 88]

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

const onResize: Resizable.PaneProps["onResize"] = (size, _previousSize) => {
  leftSize = size
}
</script>

<main class="flex flex-row flex-1 overflow-y-hidden">
  <Resizable.PaneGroup direction="horizontal" class="w-full rounded-lg bg-re" {onLayoutChange}>
    <Resizable.Pane
      {onExpand}
      {onResize}
      {onCollapse}
      maxSize={14}
      minSize={12}
      collapsible={true}
      collapsedSize={4.5}
      defaultSize={leftSize}
      class={cn('w-40 border-r border-solid border-r-secondary')}
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
    <Resizable.Pane defaultSize={rightSize} class="rounded-lg">
      <ScrollArea class="size-full flex-1">
        <slot />
      </ScrollArea>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</main>
