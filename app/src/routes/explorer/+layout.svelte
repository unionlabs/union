<script lang="ts">
import { onMount } from "svelte"
import Menu from "./(components)/menu.svelte"
import type { LayoutData } from "./$types.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import * as Resizable from "$lib/components/ui/resizable"
import GripVerticalIcon from "virtual:icons/tabler/grip-vertical"
import { ScrollArea } from "$lib/components/ui/scroll-area/index.ts"
import { page } from "$app/stores"
import { derived } from "svelte/store"
import { onNavigate } from "$app/navigation"

export let data: LayoutData

// Pane collapse on resize has been disabled because it was throwing console errors.

// let windowSize = { width: window.innerWidth, height: window.innerHeight }

// const handleResize = () => {
//   requestAnimationFrame(() => {
//     windowSize = { width: window.innerWidth, height: window.innerHeight }
//   })
// }

// onMount(() => {
//   window.addEventListener("resize", handleResize)
//   return () => {
//     window.removeEventListener("resize", handleResize)
//   }
// })

// $: if (windowSize?.width < 900) {
//   try {
//     leftPane.collapse()
//     // biome-ignore lint/suspicious/noEmptyBlockStatements: <explanation>
//   } catch {}
// }

let isCollapsed = false
let leftPane: Resizable.PaneAPI
$: [leftSize, rightSize] = [14, 88]

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

let explorerRoute = $page.route.id?.split("/").at(2)
$: explorerPageDescription = data.tables.filter(t => t.route === explorerRoute)[0].description
onNavigate(navigation => {
  if (navigation.to?.route.id?.split("/").at(1) === "explorer") {
    explorerRoute = navigation.to?.route.id?.split("/").at(2)
  }
})
</script>

<svelte:head>
  <title>Union - Explorer</title>
</svelte:head>


<main class="flex flex-row flex-1 overflow-y-hidden">
  <Resizable.PaneGroup direction="horizontal" class="w-full rounded-lg bg-re" {onLayoutChange}>
    <Resizable.Pane
      {onExpand}
      {onCollapse}
      maxSize={14}
      minSize={14}
      collapsible={true}
      collapsedSize={4.5}
      bind:pane={leftPane}
      defaultSize={leftSize}
      class={cn(
        isCollapsed ? 'min-w-13 max-w-13' : 'min-w-[180px] max-w-[180px]',
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
      <ScrollArea orientation="both" class="size-full flex-1 pr-6 pl-2">
        <h2 class="text-4xl font-bold tracking-tight mt-8 capitalize">{explorerRoute?.replaceAll('-', ' ')}</h2>
        <p class="text-muted-foreground pb-4">{explorerPageDescription}</p>
        <slot/>
      </ScrollArea>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</main>
