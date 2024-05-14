<script lang="ts">
import { routes } from "./index.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { page, navigating } from "$app/stores"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Drawer from "$lib/components/ui/drawer/index.ts"

export let navigationDrawerOpen = false
$: if ($navigating) navigationDrawerOpen = false
</script>

<Drawer.Root bind:open={navigationDrawerOpen} closeOnEscape={true} closeOnOutsideClick={true}>
  <Drawer.Trigger let:builder open></Drawer.Trigger>
  <Drawer.Content class="border-t-[1px] border-solid border-accent h-[60%] w-full min-w-full my-2">
    <nav class="flex flex-col space-y-2 justify-between h-full pt-2">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        {@const isCurrentPage = $page.route.id === path}
        <Button
          size="lg"
          href={path}
          variant="link"
          class={cn(
            draft
              ? 'hidden'
              : [
                  'rounded-none py-2 text-left text-6xl font-bold w-full hover:bg-white/5 size-full',
                  ' decoration-transparent no-underline ring-0 focus:ring-0 focus:ring-offset-0 outline-none focus-visible:outline-none focus-visible:ring-0',
                  isCurrentPage && 'bg-white/15',
                ],
          )}
        >
          {name}
        </Button>
      {/each}
    </nav>
  </Drawer.Content>
</Drawer.Root>
