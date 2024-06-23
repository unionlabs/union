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

let explorerRoute = $page.route.id?.split("/").at(2) ?? null
$: explorerPageDescription =
  data.tables.filter(t => t.route === explorerRoute).at(0)?.description ?? null

onNavigate(navigation => {
  if (navigation.to?.route.id?.split("/").at(1) === "explorer") {
    explorerRoute = navigation.to?.route.id?.split("/").at(2) ?? null
  }
})

// @ts-expect-error
$: mainExplorerPage = $page.route.id?.split("/").length <= 3
</script>

<svelte:head>
  <title>Union - Explorer</title>
</svelte:head>



<!-- mobile layout !-->
<div class="flex flex-row sm:divide-x overflow-x-none max-w-full w-full">
  <nav class={cn("sm:bg-muted h-full overflow-y-auto", explorerRoute === null ? "flex-1 sm:flex-none" : "hidden sm:block sm:w-[174px]")}>
    <h2 class="sm:hidden ml-3 mt-6 mb-3 text-2xl font-bold font-supermolot">
      Explorer
    </h2>
    <Menu tableRoutes={data.tables} isCollapsed={false} />
  </nav>
  <main class={cn("overflow-auto flex flex-col flex-1 w-full", explorerRoute === null ? "hidden sm:block" : "")}>
    <a
      class={cn(" font-bold  text-lg p-4 flex flex-row gap-2 items-center font-supermolot", 
      
      ($page.route.id?.split('/').length ?? 0) > 3 ? "" : "sm:hidden"      
      )}
      href={$page.route.id?.split('/').slice(0, -1).join('/')}
    >
      <ArrowLeftIcon />
      <span class="uppercase">{$page.route.id?.split('/').at(-2)}</span>
    </a>

    <div class="p-2 pt-0 sm:p-6 flex flex-col flex-1">
      <div class={cn($page.route.id?.split('/').length === 3 ? "" : "hidden")}>
        <h2 class="text-4xl font-extrabold font-expanded sm:!font-extra-expanded uppercase font-supermolot">
          {explorerRoute?.replaceAll('-', ' ')}
        </h2>
        <p class="pb-4 -mt-1 text-muted-foreground">{'>'} {explorerPageDescription}</p>
      </div>
      <slot />
    </div>
  </main>
</div>
