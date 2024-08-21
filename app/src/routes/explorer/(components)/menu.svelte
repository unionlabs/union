<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import type { LayoutData } from "../$types.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import { derived } from "svelte/store"

export let tableRoutes: LayoutData["tables"]

const selectedTable = derived(page, $page => $page.route.id?.split("/").at(2))

let alwaysCollapsedWidth = 580
</script>

<nav
  class={cn("rounded-none flex flex-col items-start h-full dark:bg-background")}
>
  {#each tableRoutes as { route, icon }}
    <Button
      id={route}
      title={route}
      variant="link"
      href={`/explorer/${route}`}
      class={cn(
        "mb-2 w-full flex justify-start gap-x-1",
        $selectedTable === route
          ? "bg-foreground text-primary-foreground"
          : "bg-transparent"
      )}
    >
      <span>[</span>
      <svelte:component this={icon} class={"size-3.5"} />
      <div>{route.replaceAll("-", " ")}</div>
      <span>]</span>
    </Button>
  {/each}
</nav>
