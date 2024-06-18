<script lang="ts">
import { onMount } from "svelte"
import { page } from "$app/stores"
import { onNavigate } from "$app/navigation"
import { cn } from "$lib/utilities/shadcn.ts"
import type { LayoutData } from "./$types.ts"
import Menu from "./(components)/menu.svelte"
import * as Resizable from "$lib/components/ui/resizable"
import ArrowLeftIcon from "virtual:icons/lucide/arrow-left"
import GripVerticalIcon from "virtual:icons/tabler/grip-vertical"
import { ScrollArea } from "$lib/components/ui/scroll-area/index.ts"

export let data: LayoutData

let windowSize = { width: window.innerWidth, height: window.innerHeight }

const handleResize = () => {
  requestAnimationFrame(() => {
    windowSize = { width: window.innerWidth, height: window.innerHeight }
  })
}

let isCollapsed = false
let leftPane: Resizable.PaneAPI
$: [leftSize, rightSize] = [14, 88]

onMount(() => {
  window.addEventListener("resize", handleResize)
  return () => {
    window.removeEventListener("resize", handleResize)
  }
})

$: {
  try {
    if (windowSize?.width < 900) {
      isCollapsed = true
    } else {
      isCollapsed = false
    }
    // biome-ignore lint/suspicious/noEmptyBlockStatements: <explanation>
  } catch {}
}

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

// @ts-expect-error
$: mainExplorerPage = $page.route.id?.split("/").length <= 3
</script>

<svelte:head>
  <title>Union - Explorer</title>
</svelte:head>

<main class={cn('flex flex-1 overflow-hidden', mainExplorerPage ? 'flex-row' : 'flex-col')}>
  {#if mainExplorerPage}
    <Resizable.PaneGroup
      class="w-full"
      autoSaveId="explorer"
      direction="horizontal"
      {onLayoutChange}
    >
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
          isCollapsed ? 'min-w-13 max-w-13' : 'min-w-[200px] w-[250px]',
          'w-full border-r bg-muted',
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
          <div class="py-6 pr-4 pl-2">
            <h2 class="text-4xl font-extrabold font-extra-expanded uppercase font-supermolot">
              {explorerRoute?.replaceAll('-', ' ')}
            </h2>
            <p class="pb-4 text-muted-foreground">{'>'} {explorerPageDescription}</p>
            <slot />
          </div>
        </ScrollArea>
      </Resizable.Pane>
    </Resizable.PaneGroup>
  {:else}
    <a
      class="font-bold font- text-lg p-4 flex flex-row gap-2 items-center font-supermolot"
      href={$page.route.id?.split('/').slice(0, 3).join('/')}
    >
      <ArrowLeftIcon />
      <span class="uppercase">{$page.route.id?.split('/')[2]}</span>
    </a>
    <ScrollArea class="flex-1" orientation="both">
      <div class="p-4 sm:p-6 flex items-center justify-center">
        <slot />
      </div>
    </ScrollArea>
  {/if}
</main>
