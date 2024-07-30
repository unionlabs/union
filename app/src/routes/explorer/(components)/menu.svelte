<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import type { LayoutData } from "../$types.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Tooltip from "$lib/components/ui/tooltip/index.ts"

export let tableRoutes: LayoutData["tables"]

export let isCollapsed = false

$: selectedTable = $page.route.id?.split("/").at(-1) || "transfers"

let alwaysCollapsedWidth = 580
let alwaysCollapsed = innerWidth <= alwaysCollapsedWidth
</script>

<nav
  class={cn("rounded-none flex flex-col items-start h-full dark:bg-background")}
>
  {#each tableRoutes as { route, icon }}
    {#if isCollapsed}
      <Tooltip.Root>
        <Tooltip.Trigger asChild let:builder>
          <Button
            title={route}
            variant="link"
            builders={[builder]}
            href={`/explorer/${route}`}
            data-sveltekit-reload={selectedTable === "transfers" ? true : "off"}
            class={cn(
              selectedTable === route ? "bg-muted" : "bg-transparent",
              "px-1 text-center mb-2 w-full flex py-0 hover:no-underline hover:bg-muted"
            )}
          >
            <svelte:component
              this={icon}
              class="min-w-[14px] block size-5 text-muted-foreground"
            />
            <span class="sr-only">{route}</span>
          </Button>
        </Tooltip.Trigger>

        <Tooltip.Content
          side="right"
          class="border border-solid border-accent dark:bg-card-foreground dark:text-muted"
        >
          {route}
        </Tooltip.Content>
      </Tooltip.Root>
    {:else}
      <Button
        title={route}
        variant="link"
        href={`/explorer/${route}`}
        data-sveltekit-reload={route === "transfers" ? true : "off"}
        class={cn(
          "mb-2 w-full flex justify-start gap-x-1",
          selectedTable === route
            ? "bg-foreground text-primary-foreground"
            : "bg-transparent"
        )}
      >
        <span>[</span>
        <svelte:component this={icon} class={"size-3.5"} />
        <div>{route.replaceAll("-", " ")}</div>
        <span>]</span>
      </Button>
    {/if}
  {/each}
</nav>
