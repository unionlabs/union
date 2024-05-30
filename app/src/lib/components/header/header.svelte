<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import SearchBar from "$lib/components/search-bar.svelte"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class={cn(
    'p-2 md:p-3 min-w-full w-screen flex flex-row items-center md:gap-4',
    'bg-card flex justify-between space-x-2 sm:space-x-3 border-b-[1px] border-solid border-secondary/65',
  )}
>
  <Button
    href="/"
    variant="link"
    class="p-0 mr-0 md:mr-2 no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400"
  >
    <img
      src="/images/logo/union-logo-wide-transparent.svg"
      alt="Union Logo"
      class="size-full max-w-26 select-none light:invert"
    />
  </Button>
  <div class="flex-1 p-0 m-0 w-full">
    <SearchBar />
  </div>
  <div class="hidden sm:flex flex-row space-x-0">
    <nav class="hidden lg:flex items-center space-x-0 sm:space-x-2 mr-0 sm:mr-3">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        <Button
          size="sm"
          href={path}
          variant="link"
          class={cn(
            draft
              ? 'hidden'
              : [
                  'p-2 capitalize no-underline decoration-transparent hover:outline-zinc-400/30 dark:hover:bg-zinc-800/70',
                  $page.route.id?.split('/')[1] === path.split('/')[1] && 'bg-muted-foreground/10',
                ],
          )}
        >
          {name}
        </Button>
      {/each}
    </nav>
    <div class="hidden sm:flex space-x-3">
      <Connect />
    </div>
  </div>
</header>
