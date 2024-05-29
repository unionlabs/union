<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import type { LayoutData } from "../$types.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Tooltip from "$lib/components/ui/tooltip/index.ts"

export let tableRoutes: LayoutData["tables"]

const devBorder = 0 && "outline outline-[1px] outline-pink-200/40"

export let isCollapsed = false

$: selectedTable = $page.route.id?.split("/").at(-1) || "blocks"

let innerWidth = window.innerWidth
let alwaysCollapsedWidth = 580
let alwaysCollapsed = innerWidth <= alwaysCollapsedWidth
</script>

<svelte:window bind:innerWidth />

<nav
  class={cn(
    devBorder,
    'flex flex-col items-start text-xs',
    isCollapsed || alwaysCollapsed ? 'p-1' : 'p-2',
  )}
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
            class={cn(
              selectedTable === route ? 'bg-muted' : 'bg-transparent',
              'px-1 text-center mb-2 w-full flex py-0 hover:no-underline hover:bg-muted',
            )}
          >
            <svelte:component this={icon} class="block size-5" />
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
        class={cn(
          'mb-2 w-full flex py-0 hover:no-underline hover:bg-muted px-3 text-left justify-start self-start gap-x-1',
          selectedTable === route ? 'bg-muted' : 'bg-transparent',
        )}
      >
        <svelte:component this={icon} class={'size-4.5 mr-2'} />
        <span class="block">{route}</span>
      </Button>
    {/if}
  {/each}
</nav>
