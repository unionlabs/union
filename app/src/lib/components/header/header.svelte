<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import { Badge } from "$lib/components/ui/badge/index.ts"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class={cn(
    'dark:bg-muted p-2.5 min-w-full w-screen flex flex-row items-center md:gap-4 z-10',
    'bg-card flex md:justify-start justify-between border-b border-solid',
  )}
>
  <Button
    href="/"
    variant="link"
    class="p-0 no-underline decoration-transparent border-none min-w-fit"
  >
    <img
      src="/images/logo/union-logo-supermolot.svg"
      alt="Union Logo"
      class="size-full max-w-32 h-12 select-none invert dark:invert-0"
    />
    <Badge variant="default" class="font-condensed mb-0.5 ml-1 uppercase">Testnet</Badge>
  </Button>
  <nav class="hidden md:flex items-center justify-end space-x-0 sm:space-x-2 w-full">
    {#each Object.entries(routes) as [name, { draft, path }], index (name)}
      {@const currentRoute = $page.route.id?.split('/')[1] === path.split('/').at(1)}
      <Button
        size="sm"
        href={path}
        variant="link"
        class={cn(
          draft
            ? 'hidden'
            : [
                '',
                currentRoute ? 'bg-foreground text-primary-foreground !hover:bg-foreground !hover:text-primary-foreground' : '',
              ],
        )}
      >
        {name}
      </Button>
    {/each}
  </nav>
  <div class="hidden md:flex space-x-3">
    <Connect />
  </div>
</header>
