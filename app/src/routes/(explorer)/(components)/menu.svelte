<script lang="ts">
import { page } from "$app/stores"
import { derived } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import type { LayoutData } from "../$types.ts"
import { Button } from "$lib/components/ui/button/index.ts"

export let tableRoutes: LayoutData["tables"]

const selectedTable = derived(page, $page => $page.route.id?.split("/").at(2))

let alwaysCollapsedWidth = 580
</script>

<nav class="rounded-none flex flex-col items-start h-full dark:bg-background">
  {#each tableRoutes as { name, route, icon }}
    <Button
      id={route}
      title={route}
      variant="link"
      href={`/${route}`}
      class={cn(
        'mb-2 w-full flex justify-start gap-x-1',
        $selectedTable === route ? 'bg-foreground text-primary-foreground' : 'bg-transparent',
      )}
    >
      <span>[</span>
      <svelte:component this={icon} class={'size-3.5'} />
      <div>{name}</div>
      <span>]</span>
    </Button>
  {/each}
</nav>
