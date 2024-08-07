<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import CmdK from "$lib/components/search/cmdk.svelte"
import { Badge } from "$lib/components/ui/badge/index.ts"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class={cn(
    "antialiased",
    "bg-card flex md:justify-start justify-between border-b border-solid",
    "dark:bg-muted p-2.5 min-w-full w-screen flex flex-row items-center z-10 pr-3.5"
  )}
>
  <Button
    href="/"
    variant="link"
    class="p-0 no-underline decoration-transparent border-none min-w-fit flex"
  >
    <img
      alt="Union Logo"
      src="/images/logo/union-logo-supermolot.svg"
      class="size-full max-w-30 h-9 select-none invert dark:invert-0"
    />
    <Badge class="mb-0.5 ml-1">Testnet</Badge>
  </Button>
  <div class={cn("sm:max-w-sm max-w-[30rem] w-full self-end mr-auto pl-3.25")}>
    <CmdK />
  </div>
  <nav
    class="hidden md:flex items-center justify-end space-x-0 sm:gap-x-1 w-full max-w-[18.5rem] ml-auto mx-1 pr-1"
  >
    {#each Object.entries(routes) as [name, { draft, path }], index (name)}
      {@const currentRoute =
        $page.route.id?.split("/")[1] === path.split("/").at(1)}
      <Button
        size="sm"
        href={path}
        variant="link"
        class={cn(
          draft
            ? "hidden"
            : [
                "",
                currentRoute
                  ? "bg-foreground text-primary-foreground !hover:bg-foreground !hover:text-primary-foreground"
                  : ""
              ]
        )}
      >
        {name}
      </Button>
    {/each}
  </nav>
  <div class="hidden md:flex lg:w-full lg:max-w-min max-w-[10.5rem]">
    <Connect />
  </div>
</header>
