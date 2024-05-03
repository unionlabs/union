<script lang="ts">
import { routes } from "./index.ts"
import Menu from "lucide-svelte/icons/menu"
import { cn } from "$/lib/utilities/shadcn.ts"
import { page, navigating } from "$app/stores"
import { Button } from "$/lib/components/ui/button/index.ts"
import * as Drawer from "$/lib/components/ui/drawer/index.ts"

let drawerOpen = false
$: if ($navigating) drawerOpen = false
</script>

<Drawer.Root bind:open={drawerOpen} closeOnEscape={true} closeOnOutsideClick={true}>
  <Drawer.Trigger asChild let:builder class="">
    <Button
      size="icon"
      variant="outline"
      builders={[builder]}
      class="border-none lg:hidden flex"
      on:click={() => (drawerOpen = !drawerOpen)}
    >
      <Menu class="size-9" />
    </Button>
  </Drawer.Trigger>
  <Drawer.Content class="border-t-[1px] border-solid border-accent h-[60%] w-full min-w-full my-2">
    <nav class="flex flex-col space-y-2 justify-between h-full pt-2">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        {@const isCurrentPage = $page.route.id === path}
        <Button
          size="lg"
          href={path}
          variant="link"
          class={cn([
            'rounded-none py-2 text-center text-6xl font-bold w-full hover:bg-white/5 h-full',
            ' decoration-transparent no-underline ring-0 focus:ring-0 focus:ring-offset-0 outline-none focus-visible:outline-none focus-visible:ring-0',
            { 'bg-white/15': isCurrentPage },
          ])}
        >
          {name}
        </Button>
      {/each}
    </nav>
  </Drawer.Content>
</Drawer.Root>

<style lang="postcss">
  :global([data-dialog-overlay]) {
    backdrop-filter: blur(3px);
    background-color: hsl(var(--background), 0.8);
  }
</style>
