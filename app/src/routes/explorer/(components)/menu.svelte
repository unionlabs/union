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
  class="rounded-none p-4 flex flex-col items-start h-full dark:bg-background"
>
  {#each tableRoutes as { route, icon }}
    <Button
      id={route}
      title={route}
      variant="link"
      size="sm"
      href={`/explorer/${route}`}
      class={cn(
        "rounded mb-2 w-full flex justify-start gap-x-1",
        $selectedTable === route
          ? "bg-foreground text-primary-foreground"
          : "bg-transparent"
      )}
    >
      <div>{route.replaceAll("-", " ")}</div>
    </Button>
  {/each}
</nav>
